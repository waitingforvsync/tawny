use super::component::Component;

/// MOS 6502 input pins.
pub struct Mos6502Input {
    /// Data bus (active during read cycles).
    pub data: u8,
    /// Interrupt request (active low on real hardware, active high here for clarity).
    pub irq: bool,
    /// Non-maskable interrupt.
    pub nmi: bool,
    /// When false, the CPU halts (used for DMA / cycle stretching).
    pub ready: bool,
}

/// MOS 6502 output pins.
pub struct Mos6502Output {
    /// Address bus — the address being accessed.
    pub address: u16,
    /// Data bus (active during write cycles).
    pub data: u8,
    /// true = read, false = write.
    pub rw: bool,
    /// true during an opcode fetch cycle (T1).
    pub sync: bool,
}

/// MOS 6502 CPU.
///
/// The BBC Micro uses a 6502A running at 2 MHz. It is ticked at the 4 MHz base
/// rate but only advances on CPU-phase edges (every other tick).
pub struct Mos6502 {
    // Internal registers — to be filled in during CPU implementation.
}

impl Mos6502 {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for Mos6502 {
    type Input = Mos6502Input;
    type Output = Mos6502Output;

    fn tick(&mut self, _input: &Self::Input) -> Self::Output {
        // Placeholder — returns idle bus state.
        Mos6502Output {
            address: 0,
            data: 0,
            rw: true,
            sync: false,
        }
    }

    fn reset(&mut self) {
        // Placeholder — will reset registers to power-on state.
    }
}
