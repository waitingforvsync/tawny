/// Disk drive peripheral.
///
/// Translates floppy disk controller (8271/1770) signals into reads and writes
/// against a disk image file on the host filesystem.
pub struct DiskDrive {
    // Disk image handle, track/sector state — to be filled in.
}

impl DiskDrive {
    pub fn new() -> Self {
        Self {}
    }
}
