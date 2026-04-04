pub mod disk_drive;
pub mod keyboard;
pub mod speaker;
pub mod tv;

/// A peripheral bridges between emulated hardware signals and host platform I/O.
///
/// Unlike components, peripherals don't tick at 4 MHz. They operate at their own
/// rates: the TV updates per scanline or frame, the speaker at the audio sample rate,
/// the keyboard on host input events, and the disk drive on demand.
///
/// This trait is intentionally minimal — each peripheral will define its own
/// interface methods (e.g. `write_scanline`, `poll_keys`) as we implement them.
pub trait Peripheral {
    /// The data this peripheral consumes from the emulated system.
    type Input;
    /// The data this peripheral produces back to the emulated system.
    type Output;
}
