use super::component::Component;

/// RAM input pins.
pub struct RamInput {
    /// 15-bit address (A0–A14).
    pub address: u16,
    /// Data bus.
    pub data: u8,
    /// Read/write — true = read, false = write.
    pub rw: bool,
    /// Chip enable.
    pub ce: bool,
}

/// RAM output pins.
pub struct RamOutput {
    /// Data bus (active during reads).
    pub data: u8,
}

/// 32K static RAM (IC1–IC8 on the BBC Model B).
///
/// Passive storage — no internal clock-driven state. Ticked on both video and
/// CPU phases at 4 MHz: the video circuitry and CPU take turns accessing it.
pub struct Ram {
    data: Box<[u8; 0x8000]>,
}

impl Ram {
    pub fn new() -> Self {
        Self {
            data: Box::new([0; 0x8000]),
        }
    }
}

impl Component for Ram {
    type Input = RamInput;
    type Output = RamOutput;

    fn tick(&mut self, input: &Self::Input) -> Self::Output {
        if !input.ce {
            return RamOutput { data: 0xFF };
        }
        let addr = (input.address & 0x7FFF) as usize;
        if input.rw {
            RamOutput { data: self.data[addr] }
        } else {
            self.data[addr] = input.data;
            RamOutput { data: input.data }
        }
    }

    fn reset(&mut self) {
        // RAM contents are undefined after power-on; leave as-is.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_then_read() {
        let mut ram = Ram::new();
        ram.tick(&RamInput { address: 0x1234, data: 0x42, rw: false, ce: true });
        let out = ram.tick(&RamInput { address: 0x1234, data: 0, rw: true, ce: true });
        assert_eq!(out.data, 0x42);
    }

    #[test]
    fn ce_disabled_returns_ff() {
        let mut ram = Ram::new();
        ram.tick(&RamInput { address: 0x0000, data: 0x42, rw: false, ce: true });
        let out = ram.tick(&RamInput { address: 0x0000, data: 0, rw: true, ce: false });
        assert_eq!(out.data, 0xFF);
    }

    #[test]
    fn address_wraps_to_15_bits() {
        let mut ram = Ram::new();
        ram.tick(&RamInput { address: 0x8005, data: 0xAA, rw: false, ce: true });
        let out = ram.tick(&RamInput { address: 0x0005, data: 0, rw: true, ce: true });
        assert_eq!(out.data, 0xAA);
    }
}
