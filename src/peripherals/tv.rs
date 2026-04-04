/// TV display peripheral.
///
/// Takes video output from the VIDPROC (pixel data) and CRTC (sync signals)
/// and assembles a framebuffer for wgpu to render. In future this could model
/// PAL signal characteristics: colour burst, scanline effects, phosphor persistence.
pub struct Tv {
    // Framebuffer, scanline state, etc. — to be filled in.
}

impl Tv {
    pub fn new() -> Self {
        Self {}
    }
}
