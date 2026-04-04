use super::component::Component;

/// R6522 VIA input pins.
pub struct R6522Input {
    /// Register select (4-bit, selects which internal register to access).
    pub rs: u8,
    /// Chip select (active when address decoding selects this VIA).
    pub cs: bool,
    /// Read/write signal — true = read, false = write.
    pub rw: bool,
    /// Data bus.
    pub data: u8,
}

/// R6522 VIA output pins.
pub struct R6522Output {
    /// Data bus (active during register reads).
    pub data: u8,
    /// Interrupt request output.
    pub irq: bool,
}

/// Rockwell R6522 Versatile Interface Adapter.
///
/// The BBC Micro has two VIAs:
/// - System VIA: handles keyboard scanning, sound, vertical sync interrupt,
///   speech, and the addressable latch.
/// - User VIA: handles the user port and printer.
///
/// Each VIA contains two timers, a shift register, and two 8-bit I/O ports.
/// Runs at 1 MHz (directly on the slow bus).
pub struct R6522 {
    // Internal state — to be filled in during VIA implementation.
}

impl R6522 {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for R6522 {
    type Input = R6522Input;
    type Output = R6522Output;

    fn tick(&mut self, _input: &Self::Input) -> Self::Output {
        // Placeholder — returns idle state.
        R6522Output {
            data: 0,
            irq: false,
        }
    }

    fn reset(&mut self) {
        // Placeholder — will reset to power-on state.
    }
}
