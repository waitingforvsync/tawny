/// Processor status register flag bits.
pub const C: u8 = 0x01; // Carry
pub const Z: u8 = 0x02; // Zero
pub const I: u8 = 0x04; // Interrupt disable
pub const D: u8 = 0x08; // Decimal mode
pub const B: u8 = 0x10; // Break (only exists on the stack, not in the register)
pub const U: u8 = 0x20; // Unused (always set when pushed)
pub const V: u8 = 0x40; // Overflow
pub const N: u8 = 0x80; // Negative
