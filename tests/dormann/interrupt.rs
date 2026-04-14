/// Klaus Dormann 6502 interrupt test.
///
/// Uses a feedback register at $BFFC: bit 0 = IRQ, bit 1 = NMI.
/// The test code writes to this address to trigger interrupts,
/// and the harness feeds those bits back as CPU input signals.
use std::time::Instant;
use tawny::emulator::mos6502::Mos6502;

const START_PC: u16 = 0x0400;
const SUCCESS_PC: u16 = 0x06F5;
const I_PORT: usize = 0xBFFC;
const IRQ_BIT: u8 = 0x01;
const NMI_BIT: u8 = 0x02;

#[test]
fn interrupt_test() {
    let rom = std::fs::read("tests/roms/6502_interrupt_test.bin")
        .expect("Failed to read Dormann interrupt test binary.");

    let mut ram = [0u8; 65536];
    ram[..rom.len()].copy_from_slice(&rom);

    // The feedback register must start cleared — ROM fill is $FF which
    // would trigger a spurious NMI edge on the first cycle.
    ram[I_PORT] = 0;

    let mut cpu = Mos6502::new();
    cpu.set_pc(START_PC);

    let start = Instant::now();
    let (pc, cycles) = super::run_to_trap(
        &mut cpu,
        &mut ram,
        |ram| ram[I_PORT] & IRQ_BIT != 0,
        |ram| ram[I_PORT] & NMI_BIT != 0,
    );
    let elapsed = start.elapsed();

    assert_eq!(
        pc, SUCCESS_PC,
        "Interrupt test FAILED: stuck at PC=${:04X} after {} cycles (expected ${:04X})",
        pc, cycles, SUCCESS_PC
    );
    let mhz = cycles as f64 / elapsed.as_secs_f64() / 1_000_000.0;
    println!(
        "Interrupt test PASSED at PC=${:04X} after {} cycles ({:.3}s, {:.1} MHz)",
        pc, cycles, elapsed.as_secs_f64(), mhz
    );
}
