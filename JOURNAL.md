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

## 2026-04-04 — Emulator skeleton and trait design

### What we did
- Designed the overall emulation architecture through discussion
- Created `Component` trait with typed `Input`/`Output` associated types and `tick()`/`reset()` methods
- Created `Clock` struct with 4 MHz base tick, phase tracking, and 2 MHz / 1 MHz edge helpers
- Created placeholder components: `Cpu` (6502), `Crtc` (HD6845), `Vidproc` (Video ULA), `Via` (6522)
- Created `Peripheral` trait and placeholder peripherals: `Tv`, `Keyboard`, `Speaker`, `DiskDrive`
- Created `ModelB` system with `Bus` struct and component wiring skeleton
- Added unit tests for clock phase alternation, 2 MHz edges, and 1 MHz edges

### Design decisions
- **4 MHz base tick** — captures CPU/video memory interleaving without the overhead of 16 MHz. Components that run slower use internal dividers.
- **Typed Input/Output pin structs** (params style) — `tick(&mut self, input: &Input) -> Output` chosen over mutable-fields style. Cleaner contract, easier to test, output structs are small stack values with no allocation cost.
- **Components own their pin definitions** — each component has its own Input/Output structs for true modularity. The system-level `Bus` struct is the shared representation; glue logic copies between bus and component pins.
- **Memory map as a component** — address decoding will be modelled as a component that takes an address and produces chip-select signals.
- **Peripherals separate from components** — peripherals bridge to host I/O at their own rates (frame, sample, event), not at the 4 MHz clock.
- **Newer Rust module style** — `emulator.rs` + `emulator/` folder instead of `emulator/mod.rs` to avoid ambiguous tab names in the editor.
