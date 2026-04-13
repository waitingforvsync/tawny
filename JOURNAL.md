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

## 2026-04-06 — CPU state machine refactor

### What we did
- Rewrote the micro-op model to match the Visual 6502 more closely
- `fetch_opcode` is now always the last step in every instruction's table entry — it consumes the opcode from data_latch, checks interrupts, sets tstate, increments PC, and outputs read(PC)
- phi2 is now trivial: latch data_latch, shift interrupt pipeline. No tstate decode.
- Removed `sync` field from Mos6502 struct
- Micro-ops renamed by what they consume from data_latch (e.g. `fetch_zp_addr`, `fetch_zp<OP>`, `fetch_imm<OP>`)
- ALU operations execute as soon as the operand arrives in data_latch
- Added `opcode_read` step for write-ending instructions (reads opcode from PC after a write cycle)
- Added `sync_read()` helper for opcode fetch bus outputs (sync=true)
- Separate `fetch_ind_y_hi` / `fetch_ind_y_hi_penalty` for (Indirect),Y to avoid spurious PC++
- Dormann test detection simplified to "same sync address twice"

### Performance results
- ~294 MHz (up from ~214 MHz before refactor)
- ~147x real-time for a 2 MHz BBC Micro CPU
- Cycle count increased (106M vs 96M) due to extra `opcode_read` step for write instructions, but throughput improved due to simpler micro-ops

### Design decisions
- **fetch_opcode at end of every instruction** — makes the state machine explicit and predictable. phi2 has no decode logic. The interrupt check is in one place.
- **PC increments baked into micro-ops** — each step that consumes a PC-fetched byte increments PC. No generic logic. This was the hardest part to get right — several failed attempts before finding the correct model.
- **opcode_read after writes** — write cycles leave garbage in data_latch, so an explicit read cycle is needed before fetch_opcode can decode. This adds a cycle to write instructions but matches real hardware.
- **Branch skip logic** — not-taken branches skip 3 steps to land on fetch_opcode. Taken+no-page-cross skips 2 steps. Uses direct tstate manipulation rather than cpu.next().

### Bugs encountered during refactor
- fetch_opcode initially output sync — wrong, sync belongs on the step BEFORE fetch_opcode (the one that reads the opcode from the bus)
- PC was being incremented in both fetch_opcode AND the consuming step — double increment
- Branches used cpu.next() for not-taken path, landing on branch_take instead of skipping to fetch_opcode
- Dormann test detection was checking memory patterns for BCC/BNE traps, which triggered on non-taken branches — simplified to "same sync address twice"

## 2026-04-06 — Eliminate page_crossed field; cleanup

### What we did
- Removed `page_crossed` field from Mos6502 struct. Page cross is now detected via bit 8 of `base_addr`: storing `data_latch as u16 + index as u16` naturally carries into bit 8 when a page boundary is crossed.
- `fetch_addr_hi_indexed` stores the correct address in `base_addr` and outputs the wrong-page address via `wrapping_sub(0x100)`. `fixup_indexed` simply reads from `base_addr`.
- `fetch_addr_hi_indexed_penalty` uses `wrapping_add(msb)` to preserve the carry in bit 8. `fixup_write` propagates the carry with `wrapping_add(base_addr & 0x100)`.
- Renamed `cpu.next()` → `cpu.next_state()`, added `cpu.skip_next_state()` and `cpu.inc_pc()` helpers.
- ~306 MHz release (up from ~294 MHz).

## 2026-04-06 — Consolidate duplicate micro-ops

### What we did
- Unified `fetch_imm`, `fetch_zp`, `fetch_abs` into single `fetch_data<OP>` — always does PC++ so preceding steps no longer need to. Write/RMW steps compensate with their own PC++.
- Identified and merged ~35 duplicate micro-op pairs into generic building blocks: `latch_to_base`, `read_base`, `latch_to_base_hi`, `latch_to_pc`, `latch_to_base_read_stack`, `inc_sp_read_stack`, `dummy_read`, `opcode_read`.
- Eliminated `branch_fixup` (= `opcode_read`), all per-instruction dummy/inc_sp variants (RTS/RTI/PHA/PLP shared), `jmp_abs`/`jmp_ind_hi`/`jsr_done`/`rti_read_pch`/`brk_read_vector_hi` (all = `latch_to_pc`).
- addr.rs went from ~70 functions to ~35. table.rs reads much more cleanly with generic building blocks.
- ~294 MHz release, Dormann test passes.

## 2026-04-06 — Refactor ALU operations to take explicit value parameters

### What we did
- `execute_read` now takes a `val: u8` parameter instead of reading `cpu.data_latch` directly. Call site passes `cpu.data_latch`.
- `execute_rmw` now takes a `val: u8` parameter instead of reading `cpu.data_latch`. Call sites pass the appropriate value.
- `execute_accumulator` passes `cpu.a` directly to `execute_rmw` — no more temporary data_latch hack.
- `sbc` binary path simplified: `adc_binary(cpu, !val)` instead of save/restore data_latch.
- All internal helpers (`adc`, `sbc`, `adc_binary`, `adc_decimal`, `sbc_decimal`) take `val: u8`.

## 2026-04-07 — Fix ZP indexed read timing; table helpers

### What we did
- Split ZP,X/ZP,Y read micro-ops so indexing happens during the wasted read cycle (matching real 6502 timing): `latch_to_base` → `add_index_x`/`add_index_y` → `fetch_data` → `fetch_opcode`.
- Old `index_zp_x`/`index_zp_y` retained for write, RMW, and (Indirect,X) modes.
- Added `branch_op`, `push`, `pull` const helper functions in table.rs to reduce repetition.

## 2026-04-08 — Replace const generics with ZST types and traits

### What we did
- Replaced all 42 `u8` operation constants and match-based dispatch functions with zero-sized types and trait implementations.
- Six traits: `ReadOp`, `StoreOp`, `RmwOp`, `ImpliedOp`, `PushOp`, `PullOp`.
- 42 ZSTs: `Adc`, `Sbc`, `Lda`, `Sta`, `Asl`, `Nop`, `Pha`, `Pla`, etc.
- Micro-ops in addr.rs are now generic over traits instead of `const OP: u8`, e.g. `fn fetch_data<OP: ReadOp>`.
- Table syntax simplified: `imm_read::<ops::Adc>()` instead of `imm_read::<{ops::ADC}>()`.
- ~286 MHz release, Dormann test passes.

### Design decisions
- **Trait bounds enforce valid combinations** — you can't pass `Sta` to `imm_read` because `Sta` doesn't implement `ReadOp`. The old `u8` approach had catch-all `_ => {}` arms that silently accepted any constant.
- **No performance change** — the compiler monomorphises each `<OP: Trait>` instantiation identically to the old `<const OP: u8>` approach. Both produce unique function pointers per operation.
- **PascalCase ZSTs** follow Rust naming conventions for types, making the table read naturally.

## 2026-04-08 — Fix cycle counts across the board

### What we did
- Removed superfluous dummy_read steps from push (PHA/PHP), pull (PLA/PLP), and RTI — these instructions were each one cycle too long. The "dummy read" is just fetch_opcode's bus output being ignored by the next step.
- Removed extra read_base from ZP,X/ZP,Y write modes — was 5 cycles, now correct at 4.
- Eliminated `BRK_SOFTWARE` flag. Software BRK is now `brk_flags == 0` (the natural state from fetch_opcode's normal decode path). Removed `brk_t0` step — its PC++ for the signature byte is folded into `brk_push_pch`. BRK/IRQ/NMI now 7 cycles instead of 8.
- Verified all cycle counts match documented 6502 timings for every addressing mode.
- Dormann test cycle count dropped from 106M to 96M due to corrected instruction timings.
- ~282 MHz release.

### Design decisions
- **brk_flags == 0 means software BRK** — the natural state from fetch_opcode (which clears brk_flags for normal decode) now doubles as the software BRK indicator. Only hardware interrupts set nonzero flags (IRQ, NMI, RESET). This removes a state and a step.
- **No dummy_read before push/pull/RTI** — fetch_opcode already outputs read(PC). The next micro-op simply ignores data_latch while doing productive work. The "dummy read" was an extra cycle that doesn't exist on real hardware.

## 2026-04-08 — Eliminate unnecessary state writes; merge duplicate micro-ops

### What we did
- Added `read_zp` — reads from `data_latch` directly without storing to `base_addr`. Used by `zp_read` where base_addr isn't needed later.
- Added `read_base_hi` — reads from `base_addr | (data_latch << 8)` without storing. Used by `abs_read` and `ind_x_read`.
- Merged `write_zp_indexed`, `fixup_write` into `write_base` (all identical: `PC++; write(base_addr, store_value)`). Used by ZP indexed write, absolute indexed write, and (Indirect),Y write.
- Eliminated `write_ind` — `ind_x_write` reuses `write_abs` directly (identical operation).
- Added named generators for all remaining inline step arrays: `brk()`, `jmp_abs()`, `jmp_ind()`, `jsr()`, `rts()`, `rti()`. Every opcode in the table now uses a named generator function.
- ~283 MHz release, Dormann test passes.

## 2026-04-08 — Disassembly table from addressing mode generators

### What we did
- Added `Mnemonic` enum (56 variants + `Ill`) and `AddrMode` enum (13 variants) to `mos6502.rs`.
- Added `OpEntry` struct with `mnemonic`, `addr_mode`, and a `bytes()` helper.
- Added `const MNEMONIC: Mnemonic` to all six op traits (`ReadOp`, `StoreOp`, `RmwOp`, `ImpliedOp`, `PushOp`, `PullOp`) and all implementations.
- Addressing mode generators now return `OpSteps<N>` containing both the micro-op step array and an `OpEntry`. The `set` function populates both the step table and a parallel `[OpEntry; 256]` disassembly table at compile time.
- Branch generators take an explicit `Mnemonic` parameter since branch mnemonics (BCC, BCS, etc.) aren't derivable from a trait.
- Both tables live in a single `Tables` struct built by `build_tables()`.
- Added three unit tests: spot-check of 36 opcodes, illegal opcode check, and byte count verification.
- ~287 MHz release, all tests pass.

### Design decisions
- **`MNEMONIC` on traits, not on ZSTs** — the mnemonic is associated with the trait impl, not the type itself, because some types implement multiple traits (though none do currently). This keeps the door open.
- **`OpSteps<const N: usize>`** — const generic struct lets `set` work with any step count. Each generator returns a concrete `OpSteps<2>` through `OpSteps<7>`.
- **`Ill` mnemonic for illegal opcodes** — uninitialised slots default to `ILL_ENTRY`, making it safe to index with any opcode byte.

## 2026-04-09 — Dormann test suite expansion; set_pc

### What we did
- Added `Mos6502::set_pc(pc, opcode)` — starts execution at an arbitrary address by feeding the opcode into `fetch_opcode` directly. Eliminates the reset vector patching hack.
- Reassembled all Dormann test binaries from ca65 sources (from `6502_65C02_functional_tests/ca65`).
- Added decimal mode test (`dormann_decimal.rs`) — passes.
- Added interrupt test (`dormann_interrupt.rs`) — currently `#[ignore]` due to known interrupt handling bugs.
- Split tests into separate files with a shared `helpers/mod.rs` for the `run_to_trap` harness.
- Removed the old `dormann.rs` combined test file.

### Test results
- Functional test: 96M cycles, ~305 MHz
- Decimal test: 26M cycles, ~269 MHz
- Interrupt test: initially failed — see below

## 2026-04-09 — Fix interrupt pipeline timing; all Dormann tests pass

### What we did
- Replaced `irq_latch: bool` and `nmi_latch: bool` with shift registers (`irq_shift: u8`, `nmi_shift: u8`) to model the 3-phi2 pipeline delay from the real 6502.
- phi2 shifts in `irq & !I_flag` to `irq_shift`, edge-detects NMI into `nmi_pending`, and shifts `nmi_pending` into `nmi_shift`.
- `fetch_opcode` checks bit 2 (`& 0x04`) of each shift register — this corresponds to the signal state 3 phi2 cycles ago, matching the Visual 6502 trace.
- Fixed spurious NMI in interrupt test caused by ROM fill ($FF) at the feedback register address ($BFFC) — harness now clears it before starting.
- All three Dormann tests now pass: functional, decimal, and interrupt.

### Design decisions
- **Shift registers, not booleans** — the real 6502 has a multi-stage pipeline for interrupt detection. A boolean latch can't model the delay correctly. The shift register naturally captures the pipeline depth.
- **Bit 2 = 3 phi2 delay** — fetch_opcode runs at phi1 of T2. An IRQ sampled at phi2 of cycle N needs 3 more phi2 shifts (N+1, N+2, N+3) before fetch_opcode at phi1 of cycle N+3 checks it. Bit 2 after 3 left-shifts is correct.
- **NMI edge detection then pipeline** — `nmi_pending` latches on the rising edge and stays set until serviced. It's fed through `nmi_shift` so the same pipeline delay applies.
- **Feedback register initialisation** — the Dormann interrupt test uses a memory-mapped feedback register at $BFFC. ROM fill ($FF) would assert NMI on the first cycle, so the harness clears it.

## 2026-04-09 — Eliminate rmw_result and nmi_pending fields

### What we did
- Removed `rmw_result` field. RMW ops now split into `rmw_dummy_write` (writes original back, PC++) and `rmw_execute::<OP>` (executes op on data_latch, writes result). The original value survives in data_latch because phi2 re-latches it from the dummy write.
- Removed `nmi_pending` field. Bit 0 of `nmi_shift` now acts as a sticky pending latch — set on NMI rising edge, propagated on each shift via `(nmi_shift & 1)`, cleared by fetch_opcode when the NMI is serviced.
- Mos6502 struct reduced from 15 to 13 fields.
- ~289 MHz median (up from ~272 before nmi_pending removal).
- Combined `irq_shift` and `nmi_shift` into single `int_shift: u16` — interleaved even/odd bits, shift by 2 each phi2. ~294 MHz median.
- Fixed `set_pc` — previously called `fetch_opcode` directly and discarded the bus output, meaning the first instruction's operand was never latched (data_latch contained the opcode byte instead). Now sets tstate to a dedicated `fetch_opcode` slot (last entry in step table, opcode $FF step 7) so it executes within the normal phi1/phi2 loop. This fixed the decimal test which was only running half its iterations (carry=0 only) because `LDY #$01` loaded Y with $A0 (the opcode) instead of $01.

## 2026-04-09 — All 256 opcodes implemented (legal + illegal)

### What we did
- Implemented all 105 illegal opcodes: 6 RMW combos (SLO, RLA, SRE, RRA, DCP, ISC), LAX, SAX, 8 immediate combos (ANC, ALR, ARR, ANE, LXA, AXS, USBC), unstable stores (SHA, SHX, SHY, TAS), LAS, 12 JAM/halt opcodes, and 27 NOP variants.
- RMW combos reuse existing legal op trait methods via monomorphisation — e.g. `Slo::execute` calls `Asl::execute` then ORs the result into A.
- Three new addressing mode generators: `abs_y_rmw` (7 cycles), `ind_x_rmw` (8 cycles), `ind_y_rmw` (8 cycles).
- Removed `trap()` and `ILL_ENTRY` / `Mnemonic::Ill` — every opcode slot now has a real implementation.
- JAM opcodes are explicit `set` calls (not default fill).
- All 256 opcodes covered in both step table and disassembly table. Tests verify full coverage.
- ~283 MHz, all 14 tests pass.

## 2026-04-10 — Branch interrupt latency fix

### What we did
- A 3-cycle taken branch (no page cross) has one extra cycle of interrupt latency compared to other instructions. On real hardware, the branch skips the phi2 cycle that normally samples the interrupt pipeline.
- Fixed by shifting `int_shift` right by 2 in `branch_take` when no page cross, compensating for the skipped pipeline sample. Same approach as the floooh 6502 emulation.
- Added branch-specific IRQ timing tests verifying 2-cycle (not taken), 3-cycle (taken, no page cross, extra latency), and 4-cycle (taken, page cross, normal latency) branches.
- All 17 tests pass.

## 2026-04-13 — Address translation, cycle stretching, and system wiring

### What we did
- Implemented BBC Model B address decoding: `ChipSelect` enum with 16 variants covering the full 64K address map, `decode_address()` pure function with `decode_fe_page()` helper for the 0xFE I/O page.
- Implemented `is_slow()` for 1 MHz device detection using a bits-7:5 lookup table for the 0xFE page, plus pages 0xFC/0xFD always slow.
- Created `Ram` component (32K, 15-bit address bus) and `Rom`/`Ram16k` components (16K, 14-bit address bus) with uniform `(address, data, rw, ce)` input interface.
- Created `PagedBank` enum (Rom/Ram variants) with 16 optional sideways bank slots.
- Implemented cycle stretching: phi1 output cached, phi2 deferred by 1 or 2 extra 2 MHz ticks depending on 1MHzE phase at access time. Video ticks continue during stretch.
- Restructured `ModelB::tick_cpu()` so active components (CRTC, VIAs, Vidproc) tick every CPU phase with `cs` from address decode, while passive components (RAM, ROM) only tick when selected via `route_passive()`.
- `ModelB::new()` embeds OS 1.20 and BASIC 2 ROMs via `include_bytes!`. BASIC in bank 15, sideways RAM in bank 14, paged_select defaults to 15.
- `update(cycles_2mhz)` provides the external API, internally processing pairs of 4 MHz ticks.
- Added clock signal diagram and cycle stretching reference to CLAUDE.md.
- 34 tests pass including integration test verifying CPU fetches reset vector from OS ROM.

### Design decisions
- **ChipSelect enum** — one device selected at a time mirrors hardware's active-high select lines. Pure function matches combinational decode logic.
- **Uniform Component interface** — RAM, ROM, and I/O all use the same `(address, data, rw, ce)` input shape. Passive components only ticked when selected (no wasted idle ticks). Active components tick continuously with `ce` reflecting address decode.
- **RAM ticked in both phases** — video phase uses CRTC's MA output, CPU phase uses CPU's lower 15 bits. Models the real 4 MHz interleaving.
- **Cycle stretching via deferred phi2** — phi0 stretching is modelled by caching phi1 output and delaying phi2. The CPU never sees the stretch; it just experiences a longer cycle. 2 MHz (video) continues ticking.
- **ROMs embedded at compile time** — `include_bytes!` for OS 1.20 and BASIC 2. No runtime file loading needed for the core system.

### Files changed
- `src/emulator/ram.rs` (new) — Ram component
- `src/emulator/rom.rs` (new) — Rom, Ram16k components
- `src/emulator.rs` — registered ram, rom modules
- `src/systems/model_b.rs` — complete rewrite: address decoder, memory, stretch state, glue logic, ROM loading
- `CLAUDE.md` — architecture updates, clock signal diagram
- `roms/os12.rom`, `roms/basic2.rom` (new) — ROM binaries from b2 emulator
