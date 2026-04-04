/// Keyboard peripheral.
///
/// Translates host key events (from winit) into the BBC Micro's key matrix
/// state, which the system VIA scans to detect key presses.
pub struct Keyboard {
    // Key matrix state — to be filled in.
}

impl Keyboard {
    pub fn new() -> Self {
        Self {}
    }
}
