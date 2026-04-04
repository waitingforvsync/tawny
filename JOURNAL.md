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

## 2026-04-04 — MOS 6502 CPU implementation

### What we did
- Implemented a cycle-accurate MOS 6502 CPU with all 151 legal opcodes
- Passes the Klaus Dormann 6502 functional test (96M cycles, all addressing modes, flags, decimal mode)
- Two-phase clock model: phi1 (state machine) and phi2 (data latch + interrupt pipeline)
- Static micro-op step table: 2048 entries (256 opcodes × 8 max steps), built at compile time
- Const generic dispatch for operations: `final_read::<{ops::LDA}>` monomorphises per-op
- Interrupt model: IRQ/NMI/RESET force BRK ($00) via brk_flags, no special opcode slots
- NMOS-accurate decimal mode (ADC/SBC with correct quirky flag behaviour)
- Split into sub-modules: flags.rs, ops.rs, addr.rs, table.rs
- Added lib.rs for integration test access
- Downloaded Dormann test binary, wrote integration test harness

### Design decisions
- **Opcode fetch as last step** — the last micro-op of every instruction checks the interrupt pipeline and either decodes the fetched opcode (sync → phi2 sets tstate) or forces BRK. This matches how the real 6502 works: the opcode fetch overlaps with the previous instruction's completion.
- **Interrupts as forced BRK** — no special virtual opcodes. IRQ/NMI/RESET all reuse opcode $00's microcode, with brk_flags distinguishing the source. The BRK microcode handles B flag, PC behaviour, vector selection, and write suppression (for RESET) based on these flags.
- **PC increment in phi1** — the opcode fetch micro-op increments PC. For interrupts, the increment is suppressed (PC stays pointing at the interrupted instruction).
- **data_latch timing rule** — a micro-op's bus result arrives in data_latch after phi2, available to the NEXT micro-op. This required saving branch offsets and RMW results in separate fields (base_addr, rmw_result) to survive across the phi2 boundary.
- **Const generics on u8** — Rust stable doesn't support const enum generics, so operation identifiers are u8 constants. The compiler still monomorphises each instantiation and eliminates the match at compile time.

### Bugs found and fixed during development
- `implied`/`accumulator` originally called `fetch_opcode()` internally AND had it as the next table step — instructions executed in half the expected cycles
- Branch offset was read from data_latch after a dummy read had overwritten it — saved in base_addr instead
- JMP abs table was missing the fetch_addr_hi step — jumped to wrong addresses
- RMW modify step stored result in data_latch which phi2 then overwrites — added rmw_result field
- Decimal mode ADC: N/V flags were computed incorrectly — fixed to use intermediate result after low-nybble correction but before high-nybble correction
- Decimal mode SBC: flags and BCD correction were both wrong — flags must come from binary subtraction, only the accumulator is BCD-corrected

## 2026-04-05 — CPU refactoring and performance

### What we did
- Refactored micro-ops to return `Mos6502Output` directly instead of storing intermediate state in the CPU struct. Removed `addr_bus`, `data_out`, `rw` fields.
- Added `read()` and `write()` helper constructors for `Mos6502Output`.
- Removed auto-advance logic from phi1 — every micro-op now explicitly sets its next tstate via `cpu.next()`.
- Applied DRY to table.rs: addressing mode patterns are const generic functions (`zp_read::<OP>()`, `abs_x_write::<OP>()`, etc.) returning fixed-size arrays.
- Renamed bus-setup micro-ops with `addr_` prefix for clarity (e.g. `read_zp` → `addr_zp`).
- Fixed page-cross skip bug: old auto-advance logic masked a tstate+1 that should have been tstate+2.
- Added LTO and codegen-units=1 to release profile.
- Added timing measurement to Dormann test.

### Performance results (release build, Dormann test)
- Initial fn-pointer table: ~136 MHz
- After removing intermediate state (micro-ops return output): ~226 MHz
- That's ~113x real-time for a 2 MHz BBC Micro CPU.

### Design decisions
- **Micro-ops return output** — avoids writing to self.addr_bus/data_out/rw then reading them back. Significant perf win (~65% faster) because it eliminates unnecessary stores/loads through the self pointer.
- **Explicit tstate management** — removing auto-advance eliminates a comparison per cycle and makes control flow explicit. Also fixed the page-cross skip bug that the auto-advance was masking.
- **Match vs fn-pointer table experiment** — tried a 2048-arm match dispatching through the static table. Same speed (still indirect calls through fn pointers). A true floooh-style giant switch with inlined bodies would avoid indirect calls but sacrifice readability.

### Known issues for next session
- Interrupt handling (IRQ/NMI) needs proper testing — Dormann test doesn't exercise interrupts
- The sync flag / setup_opcode_fetch logic needs cleanup
- page_crossed field could potentially be eliminated
- No unit tests for individual operations yet
