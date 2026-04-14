/// Klaus Dormann 6502 functional test.
use std::time::Instant;
use tawny::emulator::mos6502::Mos6502;

const START_PC: u16 = 0x0400;
const SUCCESS_PC: u16 = 0x3469;

#[test]
fn functional_test() {
    let rom = std::fs::read("tests/roms/6502_functional_test.bin")
        .expect("Failed to read Dormann functional test binary.");

    let mut ram = [0u8; 65536];
    ram[..rom.len()].copy_from_slice(&rom);

    let mut cpu = Mos6502::new();
    cpu.set_pc(START_PC);

    let start = Instant::now();
    let (pc, cycles) = super::run_to_trap(&mut cpu, &mut ram, |_| false, |_| false);
    let elapsed = start.elapsed();

    assert_eq!(
        pc, SUCCESS_PC,
        "Functional test FAILED: stuck at PC=${:04X} after {} cycles (expected ${:04X})",
        pc, cycles, SUCCESS_PC
    );
    let mhz = cycles as f64 / elapsed.as_secs_f64() / 1_000_000.0;
    println!(
        "Functional test PASSED at PC=${:04X} after {} cycles ({:.3}s, {:.1} MHz)",
        pc, cycles, elapsed.as_secs_f64(), mhz
    );
}
