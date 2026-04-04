use super::component::Component;

/// VLSI Video ULA (VIDPROC) input pins.
pub struct VidprocInput {
    /// Data bus (for register writes from the CPU).
    pub data: u8,
    /// Chip select (active when the CPU addresses VIDPROC).
    pub cs: bool,
    /// Video data from RAM (the byte fetched at the CRTC's memory address).
    pub video_data: u8,
    /// Display enable from CRTC.
    pub de: bool,
}

/// VLSI Video ULA (VIDPROC) output pins.
pub struct VidprocOutput {
    /// Red component of the pixel output.
    pub r: bool,
    /// Green component of the pixel output.
    pub g: bool,
    /// Blue component of the pixel output.
    pub b: bool,
}

/// VLSI Video ULA (VIDPROC).
///
/// Serialises video RAM bytes into pixel data at 8, 4, 2, or 1 pixels per byte
/// depending on the selected mode. Also handles palette lookup.
/// Clocked at 16 MHz on real hardware, but we batch pixel output at the 2 MHz rate.
pub struct Vidproc {
    // Internal state — to be filled in during VIDPROC implementation.
}

impl Vidproc {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for Vidproc {
    type Input = VidprocInput;
    type Output = VidprocOutput;

    fn tick(&mut self, _input: &Self::Input) -> Self::Output {
        // Placeholder — returns black.
        VidprocOutput {
            r: false,
            g: false,
            b: false,
        }
    }

    fn reset(&mut self) {
        // Placeholder — will reset to power-on state.
    }
}
