/// Integration test: Klaus Dormann 6502 functional test.
///
/// Loads the test binary into a flat 64K RAM, runs the CPU until PC gets
/// stuck in a `JMP *` loop, and checks whether it reached the success address.
use std::time::Instant;
use tawny::emulator::mos6502::{Mos6502, Mos6502Input, Mos6502Output};

const SUCCESS_PC: u16 = 0x3469; // Address of the success trap in the binary.

/// Simple 64K RAM bus for testing — no I/O, just flat memory.
struct TestBus {
    ram: [u8; 65536],
}

impl TestBus {
    fn new(rom: &[u8]) -> Self {
        let mut ram = [0u8; 65536];
        // The binary is a full 64K image: load it at address 0.
        ram[..rom.len()].copy_from_slice(rom);
        Self { ram }
    }

    fn access(&mut self, output: &Mos6502Output) -> u8 {
        let addr = output.address as usize;
        if output.rw {
            // Read
            self.ram[addr]
        } else {
            // Write
            self.ram[addr] = output.data;
            output.data
        }
    }
}

#[test]
fn dormann_6502_functional_test() {
    let rom = std::fs::read("tests/roms/6502_functional_test.bin")
        .expect("Failed to read Dormann test binary. Run from project root.");

    let mut cpu = Mos6502::new();
    let mut bus = TestBus::new(&rom);

    // The binary's reset vector points to the interrupt test area, not $0400.
    // Patch the reset vector to point to the actual test start.
    bus.ram[0xFFFC] = 0x00;
    bus.ram[0xFFFD] = 0x04;
    let max_cycles = 100_000_000u64;
    let mut prev_pc = 0xFFFFu16;
    let mut stuck_count = 0u32;
    let mut sync_count = 0u64;

    let start = Instant::now();
    let no_irq = Mos6502Input { data: 0, irq: false, nmi: false, ready: true };
    for cycle in 0..max_cycles {
        let output = cpu.phi1();
        let addr = output.address as usize;
        let data = if output.rw {
            bus.ram[addr]
        } else {
            bus.ram[addr] = output.data;
            output.data
        };
        cpu.phi2(&Mos6502Input { data, ..no_irq });

        if output.sync {
            sync_count += 1;
            let current_pc = output.address;
            if current_pc == prev_pc || current_pc == prev_pc.wrapping_sub(2) {
                // JMP * or BNE/BEQ * (branch to self = 2 bytes back)
                stuck_count += 1;
                if stuck_count > 3 {
                    if current_pc == SUCCESS_PC {
                        let elapsed = start.elapsed();
                        let mhz = cycle as f64 / elapsed.as_secs_f64() / 1_000_000.0;
                        println!(
                            "Dormann test PASSED at PC=${:04X} after {} cycles ({:.3}s, {:.1} MHz effective)",
                            current_pc, cycle, elapsed.as_secs_f64(), mhz
                        );
                        return;
                    } else {
                        panic!(
                            "Dormann test FAILED: stuck at PC=${:04X} after {} cycles \
                             (expected success at ${:04X})",
                            current_pc, cycle, SUCCESS_PC
                        );
                    }
                }
            } else {
                stuck_count = 0;
            }
            prev_pc = current_pc;
        }
    }

    panic!(
        "Dormann test did not complete within {} cycles ({} opcodes fetched, last sync PC=${:04X})",
        max_cycles, sync_count, prev_pc
    );
}
