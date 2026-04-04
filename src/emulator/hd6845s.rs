use super::component::Component;

/// HD6845S CRTC input pins.
pub struct Hd6845sInput {
    /// Register select (active when the CPU addresses the CRTC).
    pub rs: bool,
    /// Chip select (active when the CRTC is selected by address decoding).
    pub cs: bool,
    /// Read/write signal — true = read, false = write.
    pub rw: bool,
    /// Data bus.
    pub data: u8,
}

/// HD6845S CRTC output pins.
pub struct Hd6845sOutput {
    /// Data bus (active during register reads).
    pub data: u8,
    /// Memory address — the character address for video RAM lookup.
    pub ma: u16,
    /// Row address — the scanline within the current character row.
    pub ra: u8,
    /// Horizontal sync.
    pub hsync: bool,
    /// Vertical sync.
    pub vsync: bool,
    /// Display enable — true when the beam is in the visible area.
    pub de: bool,
}

/// Hitachi HD6845S CRT Controller.
///
/// Generates video timing and memory addresses. Runs at 1 or 2 MHz depending
/// on the video mode. Ticked at the 4 MHz base rate, uses internal dividers.
pub struct Hd6845s {
    // Internal registers — to be filled in during CRTC implementation.
}

impl Hd6845s {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for Hd6845s {
    type Input = Hd6845sInput;
    type Output = Hd6845sOutput;

    fn tick(&mut self, _input: &Self::Input) -> Self::Output {
        // Placeholder — returns idle state.
        Hd6845sOutput {
            data: 0,
            ma: 0,
            ra: 0,
            hsync: false,
            vsync: false,
            de: false,
        }
    }

    fn reset(&mut self) {
        // Placeholder — will reset registers to power-on state.
    }
}
