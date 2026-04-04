use crate::emulator::clock::Clock;
use crate::emulator::hd6845s::Hd6845s;
use crate::emulator::mos6502::Mos6502;
use crate::emulator::r6522::R6522;
use crate::emulator::vidproc::Vidproc;

/// The BBC Model B system bus.
///
/// Represents the physical lines on the motherboard. The system glue logic
/// copies values between this bus and each component's typed pin structs.
pub struct Bus {
    pub address: u16,
    pub data: u8,
    /// true = read, false = write.
    pub rw: bool,
    /// CPU interrupt request (active high — active low on real hardware, but
    /// we use active high internally for clarity).
    pub irq: bool,
    /// Non-maskable interrupt.
    pub nmi: bool,
    /// System reset.
    pub reset: bool,
}

impl Bus {
    fn new() -> Self {
        Self {
            address: 0,
            data: 0,
            rw: true,
            irq: false,
            nmi: false,
            reset: false,
        }
    }
}

/// BBC Model B system — components, bus, and glue logic.
///
/// This struct owns all the components and wires them together according to
/// the BBC Model B's motherboard schematic. The `tick` method advances the
/// entire system by one 4 MHz clock cycle.
pub struct ModelB {
    pub clock: Clock,
    pub bus: Bus,
    pub cpu: Mos6502,
    pub crtc: Hd6845s,
    pub vidproc: Vidproc,
    pub system_via: R6522,
    pub user_via: R6522,
    // Future: address decoder, RAM, ROM banks, FDC, etc.
}

impl ModelB {
    pub fn new() -> Self {
        Self {
            clock: Clock::new(),
            bus: Bus::new(),
            cpu: Mos6502::new(),
            crtc: Hd6845s::new(),
            vidproc: Vidproc::new(),
            system_via: R6522::new(),
            user_via: R6522::new(),
        }
    }

    /// Advance the system by one 4 MHz tick.
    ///
    /// This is where the motherboard glue logic lives — copying signals between
    /// the bus and component pins, deciding which components tick on this edge,
    /// and handling address decoding.
    pub fn tick(&mut self) {
        let _phase = self.clock.tick();
        // Placeholder — glue logic will go here:
        // 1. Determine which components tick on this edge
        // 2. Copy bus → component inputs
        // 3. Tick components
        // 4. Copy component outputs → bus
        // 5. Handle address decoding, interrupts, cycle stretching
    }

    /// Reset the entire system.
    pub fn reset(&mut self) {
        use crate::emulator::component::Component;
        self.cpu.reset();
        self.crtc.reset();
        self.vidproc.reset();
        self.system_via.reset();
        self.user_via.reset();
    }

    // TODO: update tick() to call cpu.phi1() and cpu.phi2() with bus routing.
}
