# Tawny - BBC Micro Emulator

## Project overview
A BBC Micro emulator written in Rust, using wgpu + winit for rendering and egui for GUI.
Named after the tawny owl — the BBC Micro logo is a stylised owl made from dots.

## Development rules
- Always write idiomatic Rust with idiomatic folder structure and naming conventions
- Explain implementation options and let the user choose
- Performance first: avoid unnecessary allocations, prefer arenas/flatmaps where appropriate
- Write tests where appropriate, ensure they pass
- Keep CLAUDE.md and JOURNAL.md up to date
- Never commit to git without user review and approval

## Tech stack
- **Language:** Rust (edition 2024)
- **Windowing:** winit 0.30 (ApplicationHandler trait)
- **Rendering:** wgpu 24 (surface clear each frame)
- **Async blocking:** pollster 0.4 (minimal blocker for wgpu's async setup)
- **GUI:** egui (not yet added)
- **Audio:** TBD

## Architecture
- `src/main.rs` — Application entry point, winit event loop, `App` struct implementing `ApplicationHandler`, `Gpu` struct holding wgpu state
- `src/emulator.rs` + `src/emulator/` — Emulation core
  - `component.rs` — `Component` trait (tick-driven, typed Input/Output pins)
  - `clock.rs` — 4 MHz master clock, phase tracking, frequency divider helpers
  - `mos6502.rs` — MOS 6502 CPU (placeholder)
  - `hd6845s.rs` — HD6845S CRT Controller (placeholder)
  - `vidproc.rs` — VLSI Video ULA (placeholder)
  - `r6522.rs` — R6522 VIA (placeholder)
- `src/peripherals.rs` + `src/peripherals/` — Host platform bridges
  - `tv.rs` — Video output to framebuffer (placeholder)
  - `keyboard.rs` — Host keys to BBC key matrix (placeholder)
  - `speaker.rs` — Audio output (placeholder)
  - `disk_drive.rs` — Disk image I/O (placeholder)
- `src/systems.rs` + `src/systems/` — System-level glue
  - `model_b.rs` — BBC Model B motherboard: Bus struct, component wiring

## Emulation design
- **Base tick rate:** 4 MHz (CPU/video memory interleaving)
- **Component model:** Each component has opaque internal state + typed Input/Output pin structs. The `Component` trait provides `tick()` and `reset()`.
- **System glue:** The `ModelB` struct owns all components and a `Bus` struct. Glue logic copies signals between the bus and component pins each tick.
- **Peripherals:** Bridge between emulated hardware and host platform. Operate at their own rates (frame, sample, event), not at 4 MHz.
- **Module convention:** Newer Rust style (`emulator.rs` + `emulator/` folder, not `mod.rs`)

## Build & run
```sh
cargo build
cargo run
```

## Tests
```sh
cargo test
```
