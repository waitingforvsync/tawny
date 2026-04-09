/// Klaus Dormann 6502 test suite.
///
/// Shared harness and submodule tests for the functional, decimal,
/// and interrupt tests.
use tawny::emulator::mos6502::{Mos6502, Mos6502Input};

#[path = "dormann/functional.rs"]
mod functional;
#[path = "dormann/decimal.rs"]
mod decimal;
#[path = "dormann/interrupt.rs"]
mod interrupt;

const MAX_CYCLES: u64 = 200_000_000;

/// Run the CPU until PC gets stuck in a JMP * loop.
/// Returns (trapped_pc, cycle_count).
fn run_to_trap(
    cpu: &mut Mos6502,
    ram: &mut [u8; 65536],
    irq_fn: impl Fn(&[u8; 65536]) -> bool,
    nmi_fn: impl Fn(&[u8; 65536]) -> bool,
) -> (u16, u64) {
    let mut prev_pc = 0xFFFFu16;
    let mut stuck_count = 0u32;

    for cycle in 0..MAX_CYCLES {
        let output = cpu.phi1();
        let addr = output.address as usize;
        let data = if output.rw {
            ram[addr]
        } else {
            ram[addr] = output.data;
            output.data
        };
        cpu.phi2(&Mos6502Input {
            data,
            irq: irq_fn(ram),
            nmi: nmi_fn(ram),
            ready: true,
        });

        if output.sync {
            let pc = output.address;
            if pc == prev_pc {
                stuck_count += 1;
                if stuck_count > 2 {
                    return (pc, cycle);
                }
            } else {
                stuck_count = 0;
            }
            prev_pc = pc;
        }
    }

    panic!(
        "Test did not complete within {} cycles (last sync PC=${:04X})",
        MAX_CYCLES, prev_pc
    );
}
