# Tawny Development Journal

## 2026-04-03 — Project bootstrap

### What we did
- Created initial project structure with `cargo init`
- Added winit 0.30 dependency
- Implemented a 1280x720 window using winit's `ApplicationHandler` trait
- Window titled "Tawny - BBC Micro Emulator"
- Application exits cleanly when window is closed

### Design decisions
- **winit ApplicationHandler trait** chosen over deprecated closure-based API — it's the modern, idiomatic approach (winit 0.30+)
- Window uses `LogicalSize` so it scales correctly on HiDPI displays

## 2026-04-03 — Added wgpu rendering

### What we did
- Added wgpu 24 and pollster 0.4 dependencies
- Created `Gpu` struct to hold wgpu surface, device, queue, and config
- Window wrapped in `Arc<Window>` (required by wgpu's `create_surface` for `'static` lifetime)
- Surface clears to dark blue each frame — needed because Wayland won't composite a window until it has drawn content
- Handle window resize by reconfiguring the surface
- Added thorough comments explaining each wgpu concept

### Design decisions
- **pollster** over tokio/async-std — we only need to block once at startup, no need for a full async runtime
- **LowPower GPU preference** — integrated graphics is fine for emulating 1980s hardware
- **sRGB surface format** — correct gamma-aware colour blending
- **Fifo present mode** — vsync, no tearing
