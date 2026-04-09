/// IRQ timing tests based on Visual 6502 traces.
///
/// Program: CLI, LDX #$80, LDA $11,X, STA $12,X, LDA $13,X, NOP, NOP
/// Data:    $0011=0B,0C,0D  $0091=BB,CC,DD
/// IRQ handler at $0020: RTI
/// IRQ vector: $FFFE=$0020
///
/// The LDA $13,X instruction finishes at Visual6502 cycle 16 (sync for
/// NOP at $0009). set_pc sets tstate so the next phi1 runs fetch_opcode,
/// so our cycle 0 = V6502 cycle 1. Our cycles = V6502 cycles - 1.
///
/// Three cases (V6502 cycle / our cycle):
/// 1. IRQ at V6502 14 / our 13 → taken instead of NOP
/// 2. IRQ at V6502 15 / our 14 → too late, NOP executes, interrupt after NOP
/// 3. IRQ at V6502 13 / our 12 → early, still taken instead of NOP
use tawny::emulator::mos6502::{Mos6502, Mos6502Input};

fn build_ram() -> [u8; 65536] {
    let mut ram = [0u8; 65536];

    // Program at $0000
    // CLI; LDX #$80; LDA $11,X; STA $12,X; LDA $13,X; NOP; NOP
    let code: &[u8] = &[
        0x58,             // $0000: CLI
        0xA2, 0x80,       // $0001: LDX #$80
        0xB5, 0x11,       // $0003: LDA $11,X
        0x95, 0x12,       // $0005: STA $12,X
        0xB5, 0x13,       // $0007: LDA $13,X
        0xEA,             // $0009: NOP
        0xEA,             // $000A: NOP
    ];
    ram[0x0000..0x0000 + code.len()].copy_from_slice(code);

    // Data in zero page
    ram[0x0011] = 0x0B;
    ram[0x0012] = 0x0C;
    ram[0x0013] = 0x0D;

    // Data at $0091-$0093 (ZP $11+$80, $12+$80, $13+$80 wrapping)
    ram[0x0091] = 0xBB;
    ram[0x0092] = 0xCC;
    ram[0x0093] = 0xDD;

    // IRQ handler: RTI
    ram[0x0020] = 0x40;

    // IRQ vector
    ram[0xFFFE] = 0x20;
    ram[0xFFFF] = 0x00;

    ram
}

/// Run CPU for a fixed number of cycles, asserting IRQ from `irq_start`
/// through `irq_end` (inclusive). Returns a Vec of (cycle, pc) for every sync.
fn run_with_irq(irq_start: u64, irq_end: u64, total_cycles: u64) -> Vec<(u64, u16)> {
    let mut ram = build_ram();
    let mut cpu = Mos6502::new();
    cpu.set_pc(0x0000, ram[0x0000]);

    let mut syncs = Vec::new();

    for cycle in 0..total_cycles {
        let output = cpu.phi1();
        let addr = output.address as usize;
        let data = if output.rw {
            ram[addr]
        } else {
            ram[addr] = output.data;
            output.data
        };

        if output.sync {
            syncs.push((cycle, output.address));
        }

        cpu.phi2(&Mos6502Input {
            data,
            irq: cycle >= irq_start && cycle <= irq_end,
            nmi: false,
            ready: true,
        });
    }

    syncs
}

/// Check whether the IRQ handler at $0020 appears between the first and
/// second sync at $0009. If it does, the interrupt was taken instead of NOP.
/// If not, the NOP executed first and the interrupt was deferred.
fn irq_taken_before_nop(syncs: &[(u64, u16)]) -> bool {
    // Find the opcode_read sync at $0009 (first occurrence).
    let first_0009 = syncs.iter().position(|(_, p)| *p == 0x0009)
        .expect("should see sync at $0009");
    // Find the IRQ handler sync at $0020.
    let handler = syncs.iter().position(|(_, p)| *p == 0x0020)
        .expect("should see IRQ handler at $0020");
    // If the handler appears right after the first $0009 sync,
    // the interrupt was taken instead of NOP.
    handler == first_0009 + 1
}

#[test]
fn irq_at_last_possible_cycle_takes_interrupt_instead_of_nop() {
    // IRQ asserted at V6502 cycle 14 (our cycle 13) — the last cycle where
    // the interrupt can be taken instead of NOP. Held through the pipeline.
    let syncs = run_with_irq(13, 15, 40);
    assert!(irq_taken_before_nop(&syncs),
        "IRQ at V6502 cycle 14 should be taken instead of NOP");
}

#[test]
fn irq_one_cycle_too_late_deferred_past_nop() {
    // IRQ asserted at V6502 cycle 15 (our cycle 14) — one cycle too late.
    // NOP executes normally, interrupt taken after NOP.
    let syncs = run_with_irq(14, 16, 40);
    assert!(!irq_taken_before_nop(&syncs),
        "IRQ at V6502 cycle 15 should be deferred past NOP");
}

#[test]
fn irq_one_cycle_early_still_takes_interrupt() {
    // IRQ asserted at V6502 cycle 13 (our cycle 12) — one cycle earlier,
    // still taken instead of NOP.
    let syncs = run_with_irq(12, 15, 40);
    assert!(irq_taken_before_nop(&syncs),
        "IRQ at V6502 cycle 13 should still be taken instead of NOP");
}
