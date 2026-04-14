/// Klaus Dormann 6502 decimal mode test.
use std::time::Instant;
use tawny::emulator::mos6502::Mos6502;

const START_PC: u16 = 0x0200;
const DONE_PC: u16 = 0x025A;
const ERROR_ADDR: u16 = 0x000B;

#[test]
fn decimal_test() {
    let rom = std::fs::read("tests/roms/6502_decimal_test.bin")
        .expect("Failed to read Dormann decimal test binary.");

    let mut ram = [0u8; 65536];
    ram[..rom.len()].copy_from_slice(&rom);

    let mut cpu = Mos6502::new();
    cpu.set_pc(START_PC);

    let start = Instant::now();
    let (pc, cycles) = super::run_to_trap(&mut cpu, &mut ram, |_| false, |_| false);
    let elapsed = start.elapsed();

    assert_eq!(
        pc, DONE_PC,
        "Decimal test FAILED: stuck at PC=${:04X} after {} cycles (expected ${:04X})",
        pc, cycles, DONE_PC
    );
    assert_eq!(
        ram[ERROR_ADDR as usize], 0,
        "Decimal test FAILED: ERROR flag is {} (expected 0)",
        ram[ERROR_ADDR as usize]
    );
    let mhz = cycles as f64 / elapsed.as_secs_f64() / 1_000_000.0;
    println!(
        "Decimal test PASSED after {} cycles ({:.3}s, {:.1} MHz)",
        cycles, elapsed.as_secs_f64(), mhz
    );
}
