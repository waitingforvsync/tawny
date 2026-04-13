use super::component::Component;

/// ROM input pins.
pub struct RomInput {
    /// 14-bit address (A0–A13).
    pub address: u16,
    /// Data bus (ignored — ROM is read-only).
    pub data: u8,
    /// Read/write — true = read, false = write (writes are ignored).
    pub rw: bool,
    /// Chip enable.
    pub ce: bool,
}

/// ROM output pins.
pub struct RomOutput {
    /// Data bus (active during reads).
    pub data: u8,
}

/// 16K ROM (OS ROM or sideways ROM/RAM slot).
///
/// Passive storage — no internal clock-driven state. Writes are silently
/// ignored for ROM; the `Ram16k` variant provides writable sideways RAM.
pub struct Rom {
    data: Box<[u8; 0x4000]>,
}

impl Rom {
    pub fn new(data: Box<[u8; 0x4000]>) -> Self {
        Self { data }
    }
}

impl Component for Rom {
    type Input = RomInput;
    type Output = RomOutput;

    fn tick(&mut self, input: &Self::Input) -> Self::Output {
        if !input.ce {
            return RomOutput { data: 0xFF };
        }
        let addr = (input.address & 0x3FFF) as usize;
        RomOutput { data: self.data[addr] }
    }

    fn reset(&mut self) {
        // ROM contents are fixed; nothing to reset.
    }
}

/// 16K sideways RAM — same interface as ROM but supports writes.
pub struct Ram16k {
    data: Box<[u8; 0x4000]>,
}

impl Ram16k {
    pub fn new() -> Self {
        Self {
            data: Box::new([0; 0x4000]),
        }
    }
}

impl Component for Ram16k {
    type Input = RomInput;
    type Output = RomOutput;

    fn tick(&mut self, input: &Self::Input) -> Self::Output {
        if !input.ce {
            return RomOutput { data: 0xFF };
        }
        let addr = (input.address & 0x3FFF) as usize;
        if input.rw {
            RomOutput { data: self.data[addr] }
        } else {
            self.data[addr] = input.data;
            RomOutput { data: input.data }
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
    fn rom_reads_data() {
        let mut data = Box::new([0u8; 0x4000]);
        data[0x3FFC] = 0x00;
        data[0x3FFD] = 0xC0;
        let mut rom = Rom::new(data);
        let out = rom.tick(&RomInput { address: 0x3FFC, data: 0, rw: true, ce: true });
        assert_eq!(out.data, 0x00);
        let out = rom.tick(&RomInput { address: 0x3FFD, data: 0, rw: true, ce: true });
        assert_eq!(out.data, 0xC0);
    }

    #[test]
    fn rom_ignores_writes() {
        let mut rom = Rom::new(Box::new([0u8; 0x4000]));
        rom.tick(&RomInput { address: 0x0000, data: 0x42, rw: false, ce: true });
        let out = rom.tick(&RomInput { address: 0x0000, data: 0, rw: true, ce: true });
        assert_eq!(out.data, 0x00);
    }

    #[test]
    fn ram16k_writes_and_reads() {
        let mut ram = Ram16k::new();
        ram.tick(&RomInput { address: 0x1000, data: 0xBE, rw: false, ce: true });
        let out = ram.tick(&RomInput { address: 0x1000, data: 0, rw: true, ce: true });
        assert_eq!(out.data, 0xBE);
    }

    #[test]
    fn ce_disabled_returns_ff() {
        let mut rom = Rom::new(Box::new([0x42; 0x4000]));
        let out = rom.tick(&RomInput { address: 0x0000, data: 0, rw: true, ce: false });
        assert_eq!(out.data, 0xFF);
    }

    #[test]
    fn address_wraps_to_14_bits() {
        let mut data = Box::new([0u8; 0x4000]);
        data[0x0005] = 0x77;
        let mut rom = Rom::new(data);
        let out = rom.tick(&RomInput { address: 0xC005, data: 0, rw: true, ce: true });
        assert_eq!(out.data, 0x77);
    }
}
