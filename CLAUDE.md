# Tawny - BBC Micro Emulator

## Project overview
A BBC Micro emulator written in Rust, using wgpu + winit for rendering and egui for GUI.
Named after the tawny owl — the BBC Micro logo is a stylised owl made from dots.

## Development rules
- Always write idiomatic Rust with idiomatic folder structure and naming conventions
- Explain implementation options and let the user choose
- Performance first: avoid unnecessary allocations, prefer arenas/flatmaps where appropriate
- Favour value semantics where there's no big performance penalty, avoid mutable variables
- Write tests where appropriate, ensure they pass
- Keep CLAUDE.md and JOURNAL.md up to date
- Never commit or push to git without explicit user approval — always wait for the user to review changes first, even when asked to "update docs, commit and push" in a single request. Present the changes for review before executing git commands.

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
  - `mos6502.rs` + `mos6502/` — MOS 6502 CPU (cycle-accurate, passes Dormann test)
    - `flags.rs` — Processor status flag bit constants
    - `ops.rs` — ALU/register operations via ZST types and traits (legal + illegal)
    - `addr.rs` — Addressing mode micro-op functions
    - `table.rs` — Compile-time step table (all 256 opcodes × 8 steps) + disassembly table
  - `ram.rs` — 32K static RAM (Component with 15-bit address, data, rw, ce pins)
  - `rom.rs` — 16K ROM + 16K sideways RAM (`Ram16k`), both with 14-bit address pins
  - `hd6845s.rs` — HD6845S CRT Controller (placeholder)
  - `vidproc.rs` — VLSI Video ULA (placeholder)
  - `r6522.rs` — R6522 VIA (placeholder)
- `src/peripherals.rs` + `src/peripherals/` — Host platform bridges
  - `tv.rs` — Video output to framebuffer (placeholder)
  - `keyboard.rs` — Host keys to BBC key matrix (placeholder)
  - `speaker.rs` — Audio output (placeholder)
  - `disk_drive.rs` — Disk image I/O (placeholder)
- `src/systems.rs` + `src/systems/` — System-level glue
  - `model_b.rs` — BBC Model B motherboard: address decoder, cycle stretching, component wiring, ROM loading
- `roms/` — ROM binaries (OS 1.20, BASIC 2) embedded via `include_bytes!`

## Emulation design
- **Base tick rate:** 4 MHz (CPU/video memory interleaving)
- **Component model:** Each component has opaque internal state + typed Input/Output pin structs. The `Component` trait provides `tick()` and `reset()`. All devices (including RAM/ROM) share the same input shape: address, data, rw, chip enable (ce).
- **Active vs passive components:** Active components (VIAs, CRTC, Vidproc) tick every CPU phase regardless of chip select — they have internal state (timers, counters) that must advance continuously. Passive components (RAM, ROM) are only ticked when selected by address decoding.
- **System glue:** `ModelB` owns all components. `update(cycles_2mhz)` processes 2 MHz timeslices, internally ticking at 4 MHz. Address decoding is a pure function returning a `ChipSelect` enum. Active components are ticked first, then the address decode result routes data to/from the appropriate device.
- **Address map:** `ChipSelect` enum covers RAM (0x0000–0x7FFF), paged ROM/RAM (0x8000–0xBFFF, 16 banks), OS ROM (0xC000–0xFBFF, 0xFF00–0xFFFF), and 12 I/O devices in the 0xFC00–0xFEFF region.
- **Cycle stretching:** 1 MHz device accesses stretch phi0 by holding it high. Two shapes depending on 1MHzE phase: 1 or 2 extra 2 MHz ticks. Implemented by caching phi1 output and deferring phi2. Video ticks continue during the stretch.
- **Peripherals:** Bridge between emulated hardware and host platform. Operate at their own rates (frame, sample, event), not at 4 MHz.
- **Module convention:** Newer Rust style (`emulator.rs` + `emulator/` folder, not `mod.rs`)

## 6502 CPU design
- **Two-phase clock:** phi1 dispatches micro-op (returns bus output), phi2 latches data + shifts interrupt pipeline
- **Step table:** Static array of `fn(&mut Mos6502) -> Mos6502Output` pointers indexed by `(opcode << 3) | step`
- **Micro-ops named by what they consume from data_latch** — e.g. `fetch_zp_addr` consumes a ZP address, `fetch_zp<LDA>` consumes a ZP value and executes LDA
- **`fetch_opcode` is always the last step** — it consumes the opcode from data_latch, checks interrupts, sets tstate (or forces BRK), increments PC, outputs read(PC)
- **phi2 is trivial** — just latches data_latch and shifts the interrupt pipeline. No decode logic.
- **ALU ops execute as soon as operand arrives** — the step that has the operand in data_latch executes the operation immediately, then outputs sync_read(PC) for the next opcode
- **PC increments baked into each micro-op** — no generic logic. Steps that consume a PC-fetched byte increment PC; steps that consume from computed addresses don't.
- **`opcode_read` step** after write cycles — reads the next opcode from PC (since data_latch after a write contains the written value, not an opcode)
- **Interrupts via forced BRK:** IRQ/NMI/RESET all use opcode $00's microcode, distinguished by `brk_flags` (0 = software BRK, nonzero = interrupt). No special opcode slots.
- **ZST trait dispatch** for operations: each op is a zero-sized type (`ops::Lda`, `ops::Adc`, etc.) implementing a trait (`ReadOp`, `StoreOp`, `RmwOp`, `ImpliedOp`, `PushOp`, `PullOp`). Micro-ops are generic over the trait: `fetch_data::<ops::Lda>` monomorphises into a unique function pointer per operation, with compile-time enforcement that only valid ops are used with each addressing mode.
- **Micro-ops return output directly** — no intermediate state fields on the CPU struct. Read-only modes use stateless micro-ops (`read_zp`, `read_base_hi`) that skip writing to `base_addr`; write modes share `write_base` for the final store.
- **Page cross detection** uses bit 8 of `base_addr` — the u16 result of `data_latch + index` naturally carries into bit 8 when a page boundary is crossed. No separate `page_crossed` field needed.
- **All cycle counts match documented 6502 timings** — verified against reference for every addressing mode
- **Interrupt pipeline:** Combined `int_shift: u16` shift register — even bits = IRQ, odd bits = NMI, shifts left by 2 each phi2. `fetch_opcode` checks bit 5 (NMI) and bit 4 (IRQ). NMI is edge-detected (`nmi_prev`), with bit 1 of `int_shift` acting as a sticky pending latch. `branch_take` shifts right by 2 on no-page-cross to model the extra cycle of interrupt latency for 3-cycle taken branches. Passes Dormann interrupt test + IRQ timing tests.
- **Visual 6502 reference** http://www.visual6502.org/JSSim/expert.html?graphics=false&steps=40&a=0000&d=58a5088509a50aea69674240&a=FFFE&d=0b00&r=0000&loglevel=3&logmore=idl,irq,sync,abl,abh&irq0=19
```
cycle	ab	db	rw	Fetch	pc	a	x	y	s	p	Execute	State	ir	tcstate	pd	idl	irq	sync	abl	abh
0	0000	58	1	CLI	0000	aa	00	00	fd	nv‑BdIZc	BRK	T1	00	101111	00	00	1	1	00	00
0	0000	58	1	CLI	0000	aa	00	00	fd	nv‑BdIZc	BRK	T1	00	101111	58	58	1	1	00	00
1	0001	a5	1		0001	aa	00	00	fd	nv‑BdIZc	CLI	T0+T2	58	010111	58	58	1	0	01	00
1	0001	a5	1		0001	aa	00	00	fd	nv‑BdIZc	CLI	T0+T2	58	010111	a5	a5	1	0	01	00
2	0001	a5	1	LDA zp	0001	aa	00	00	fd	nv‑BdiZc	CLI	T1	58	101111	a5	a5	1	1	01	00
2	0001	a5	1	LDA zp	0001	aa	00	00	fd	nv‑BdiZc	CLI	T1	58	101111	a5	a5	1	1	01	00
3	0002	08	1		0002	aa	00	00	fd	nv‑BdiZc	LDA zp	T2	a5	110111	a5	a5	1	0	02	00
3	0002	08	1		0002	aa	00	00	fd	nv‑BdiZc	LDA zp	T2	a5	110111	08	08	1	0	02	00
4	0008	69	1		0003	aa	00	00	fd	nv‑BdiZc	LDA zp	T0	a5	011111	08	08	1	0	08	00
4	0008	69	1		0003	aa	00	00	fd	nv‑BdiZc	LDA zp	T0	a5	011111	69	69	1	0	08	00
5	0003	85	1	STA zp	0003	69	00	00	fd	nv‑Bdizc	LDA zp	T1	a5	101111	69	69	1	1	03	00
5	0003	85	1	STA zp	0003	69	00	00	fd	nv‑Bdizc	LDA zp	T1	a5	101111	85	85	1	1	03	00
6	0004	09	1		0004	69	00	00	fd	nv‑Bdizc	STA zp	T2	85	110111	85	85	1	0	04	00
6	0004	09	1		0004	69	00	00	fd	nv‑Bdizc	STA zp	T2	85	110111	09	09	1	0	04	00
7	0009	09	0		0005	69	00	00	fd	nv‑Bdizc	STA zp	T0	85	011111	09	09	1	0	09	00
7	0009	69	0		0005	69	00	00	fd	nv‑Bdizc	STA zp	T0	85	011111	69	69	1	0	09	00
8	0005	a5	1	LDA zp	0005	69	00	00	fd	nv‑Bdizc	STA zp	T1	85	101111	69	69	1	1	05	00
8	0005	a5	1	LDA zp	0005	69	00	00	fd	nv‑Bdizc	STA zp	T1	85	101111	a5	a5	1	1	05	00
9	0006	0a	1		0006	69	00	00	fd	nv‑Bdizc	LDA zp	T2	a5	110111	a5	a5	1	0	06	00
9	0006	0a	1		0006	69	00	00	fd	nv‑Bdizc	LDA zp	T2	a5	110111	0a	0a	0	0	06	00
10	000a	42	1		0007	69	00	00	fd	nv‑Bdizc	LDA zp	T0	a5	011111	0a	0a	0	0	0a	00
10	000a	42	1		0007	69	00	00	fd	nv‑bdizc	LDA zp	T0	a5	011111	42	42	0	0	0a	00
11	0007	ea	1	NOP	0007	42	00	00	fd	nv‑bdizc	LDA zp	T1	a5	101111	42	42	0	1	07	00
11	0007	ea	1	NOP	0007	42	00	00	fd	nv‑bdizc	LDA zp	T1	a5	101111	ea	ea	0	1	07	00
12	0007	ea	1		0007	42	00	00	fd	nv‑bdizc	BRK	T2	00	110111	ea	ea	0	0	07	00
12	0007	ea	1		0007	42	00	00	fd	nv‑bdizc	BRK	T2	00	110111	ea	ea	0	0	07	00
13	01fd	ea	0		0007	42	00	00	fd	nv‑bdizc	BRK	T3	00	111011	ea	ea	0	0	fd	01
13	01fd	00	0		0007	42	00	00	fd	nv‑bdizc	BRK	T3	00	111011	00	00	0	0	fd	01
14	01fc	ea	0		0007	42	00	00	fd	nv‑bdizc	BRK	T4	00	111101	00	00	0	0	fc	01
14	01fc	07	0		0007	42	00	00	fd	nv‑bdizc	BRK	T4	00	111101	07	07	0	0	fc	01
15	01fb	ea	0		0007	42	00	00	fd	nv‑bdizc	BRK	T5	00	111110	07	07	0	0	fb	01
15	01fb	20	0		0007	42	00	00	fd	nv‑bdizc	BRK	T5	00	111110	20	20	0	0	fb	01
16	fffe	0b	1		0007	42	00	00	fa	nv‑bdizc	BRK		00	111111	20	20	0	0	fe	ff
16	fffe	0b	1		0007	42	00	00	fa	nv‑Bdizc	BRK		00	111111	0b	0b	0	0	fe	ff
17	ffff	00	1		0007	42	00	00	fa	nv‑BdIzc	BRK	T0	00	011111	0b	0b	0	0	ff	ff
17	ffff	00	1		0007	42	00	00	fa	nv‑BdIzc	BRK	T0	00	011111	00	00	0	0	ff	ff
18	000b	40	1	RTI	000b	42	00	00	fa	nv‑BdIzc	BRK	T1	00	101111	00	00	0	1	0b	00
18	000b	40	1	RTI	000b	42	00	00	fa	nv‑BdIzc	BRK	T1	00	101111	40	40	0	1	0b	00
19	000c	00	1		000c	42	00	00	fa	nv‑BdIzc	RTI	T2	40	110111	40	40	0	0	0c	00
19	000c	00	1		000c	42	00	00	fa	nv‑BdIzc	RTI	T2	40	110111	00	00	0	0	0c	00
```

## BBC Model B clock signals and cycle stretching

The BBC Micro has a 16 MHz crystal divided down. The key clock signals:

```
4 MHz           -_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_
2 MHz           --__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__
1 MHzE          ----____----____----____----____----____----____----____----____
1 MHz device    ____----________________________----____________________________
phi0            __--__------__--__--__--__--__--______------__--__--__--__--__--
phi1            --__--______--__--__--__--__--__------______--__--__--__--__--__
2 MHzE (phi2)   __--__------__--__--__--__--__--______------__--__--__--__--__--
```

- **4 MHz**: divided master clock. Each char = one half-cycle (125 ns).
- **2 MHz**: 4 MHz ÷ 2. Drives RAM access alternation (video/CPU interleaving). Runs continuously, never stretched.
- **1 MHzE**: 2 MHz ÷ 2. Clock signal to 1 MHz devices (VIAs etc).
- **1 MHz device**: output from address decode glue logic, high when a 1 MHz device is being accessed.
- **phi0**: clock input to the 6502. Normally follows 2 MHz. Discrete glue logic (ICs 23, 29, 30, 31, 33, 34, 38) stretches phi0 by holding it high or low when a 1 MHz device is accessed, until 1 MHz and 2 MHz falling edges align.
- **phi1**: derived by the 6502 as inverse of phi0.
- **2 MHzE**: derived from phi1 (inverted). Clocks 2 MHz devices. Also stretched.

**Two stretch shapes** (shown in the diagram):
- **1MHzE low at access time**: phi0 high extended from 2→6 half-cycles of 4 MHz. Stretched cycle = 8 half-cycles total. Costs **1 extra 2 MHz tick**.
- **1MHzE high at access time**: phi0 low AND high both extended to 6 half-cycles. Stretched cycle = 12 half-cycles total. Costs **2 extra 2 MHz ticks**.

During stretching, 2 MHz continues normally — video reads from RAM keep ticking. Only the CPU (and 2 MHzE-clocked devices) are frozen.

**Note**: The BBC Micro does NOT have bus contention in higher video modes. The Acorn Electron does.

## Build & run
```sh
cargo build
cargo run
```

## Tests
```sh
cargo test
```
