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

## Build & run
```sh
cargo build
cargo run
```

## Tests
```sh
cargo test
```
