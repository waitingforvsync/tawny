mod emulator;
mod peripherals;
mod systems;

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

/// GPU state: everything wgpu needs to render frames.
/// Kept separate from App so we can initialise it only once the window exists.
struct Gpu {
    /// The drawable surface tied to our window — this is what we render into.
    surface: wgpu::Surface<'static>,
    /// Logical handle to the GPU. Used to create resources (buffers, textures, encoders).
    device: wgpu::Device,
    /// Command submission queue. We send encoded GPU commands here each frame.
    queue: wgpu::Queue,
    /// Current surface configuration (format, size, present mode). Updated on resize.
    config: wgpu::SurfaceConfiguration,
}

/// Top-level application state.
/// Both fields are Option because on some platforms (e.g. Android) the window
/// isn't available until the `resumed` event fires.
struct App {
    gpu: Option<Gpu>,
    /// Arc because wgpu's create_surface requires a 'static reference to the window.
    /// Arc gives us shared ownership with a 'static lifetime.
    window: Option<Arc<Window>>,
}

impl App {
    fn new() -> Self {
        Self {
            gpu: None,
            window: None,
        }
    }

    /// Set up wgpu: create an instance, surface, adapter, device, and configure the surface.
    fn init_gpu(&mut self, window: Arc<Window>) {
        // Instance is the entry point to wgpu. Backends::all() picks Vulkan, Metal, DX12,
        // or OpenGL depending on the platform.
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // The surface is the bridge between the window and the GPU — it provides
        // the textures we render into, which the OS then composites on screen.
        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");

        // An adapter represents a physical GPU. We ask for one that's compatible with
        // our surface. LowPower prefers integrated graphics (no need for a discrete GPU
        // to emulate 1980s hardware). pollster::block_on blocks the current thread on
        // the async result — fine here since we only do this once at startup.
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("Failed to find a suitable GPU adapter");

        // A device is a logical connection to the GPU. The queue is where we submit
        // command buffers for the GPU to execute.
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("tawny"),
                ..Default::default()
            },
            None,
        ))
        .expect("Failed to create device");

        // Query the surface's supported formats and pick an sRGB one if available.
        // sRGB gives us correct gamma-aware colour blending.
        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        // Configure the surface: what size, format, and presentation mode to use.
        // Fifo = vsync (present at display refresh rate, no tearing).
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        self.window = Some(window);
        self.gpu = Some(Gpu {
            surface,
            device,
            queue,
            config,
        });
    }

    /// Render a single frame. Currently just clears the screen.
    fn render(&self) {
        let gpu = self.gpu.as_ref().unwrap();

        // Get the next texture from the swapchain to render into.
        let output = match gpu.surface.get_current_texture() {
            Ok(tex) => tex,
            // Lost = surface needs reconfiguring (e.g. after a resize race). Skip this frame.
            Err(wgpu::SurfaceError::Lost) => return,
            Err(wgpu::SurfaceError::OutOfMemory) => {
                eprintln!("Out of GPU memory");
                return;
            }
            Err(e) => {
                eprintln!("Surface error: {e}");
                return;
            }
        };

        // Create a view into the texture — this is what the render pass writes to.
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // A command encoder records GPU commands into a command buffer.
        // Nothing executes until we submit the finished buffer to the queue.
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render"),
            });

        // A render pass is a sequence of draw commands targeting one or more attachments.
        // Here we just clear to dark blue.
        // LoadOp::Clear fills the attachment with the given colour before any drawing.
        // StoreOp::Store keeps the result (as opposed to Discard for transient passes).
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.2,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        // The render pass must be dropped before we can call encoder.finish(),
        // because it borrows the encoder mutably.
        drop(_pass);

        // Submit the command buffer and present the frame to the screen.
        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    /// Reconfigure the surface when the window is resized.
    /// Guards against zero-size (which happens when the window is minimised).
    fn resize(&mut self, width: u32, height: u32) {
        if let Some(gpu) = &mut self.gpu {
            if width > 0 && height > 0 {
                gpu.config.width = width;
                gpu.config.height = height;
                gpu.surface.configure(&gpu.device, &gpu.config);
            }
        }
    }
}

/// winit's ApplicationHandler trait — the modern (0.30+) way to structure a winit app.
/// The event loop calls these methods on our App struct in response to OS events.
impl ApplicationHandler for App {
    /// Called when the application is ready to create windows and initialise graphics.
    /// On desktop this fires once at startup; on mobile it can fire multiple times
    /// (e.g. after the app returns from the background).
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            // LogicalSize means 1280x720 in logical pixels — on HiDPI displays the
            // physical pixel count will be larger, but the window appears the same size.
            let attrs = Window::default_attributes()
                .with_title("Tawny - BBC Micro Emulator")
                .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

            match event_loop.create_window(attrs) {
                Ok(window) => {
                    let window = Arc::new(window);
                    self.init_gpu(window);
                }
                Err(e) => {
                    eprintln!("Failed to create window: {e}");
                    event_loop.exit();
                }
            }
        }
    }

    /// Called for every event on our window.
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                if self.gpu.is_some() {
                    self.render();
                }
                // Request another redraw immediately — this gives us a continuous render loop.
                // Later, we may want to throttle this to the emulator's frame rate.
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    // EventLoop is the OS event pump. run_app blocks until the app exits.
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("Event loop error");
}
