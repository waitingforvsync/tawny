pub mod addr;
pub mod flags;
pub mod ops;
pub mod table;

use flags::*;

pub const MAX_STEPS: usize = 8;
pub const TABLE_SIZE: usize = 256 * MAX_STEPS;

pub type MicroOp = fn(&mut Mos6502) -> Mos6502Output;

// ======================================================================
// Disassembly metadata
// ======================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mnemonic {
    Adc, And, Asl, Bcc, Bcs, Beq, Bit, Bmi, Bne, Bpl, Brk, Bvc, Bvs,
    Clc, Cld, Cli, Clv, Cmp, Cpx, Cpy,
    Dec, Dex, Dey,
    Eor,
    Inc, Inx, Iny,
    Jmp, Jsr,
    Lda, Ldx, Ldy, Lsr,
    Nop,
    Ora,
    Pha, Php, Pla, Plp,
    Rol, Ror, Rti, Rts,
    Sbc, Sec, Sed, Sei, Sta, Stx, Sty,
    Tax, Tay, Tsx, Txa, Txs, Tya,
    Ill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddrMode {
    Implied,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    Relative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpEntry {
    pub mnemonic: Mnemonic,
    pub addr_mode: AddrMode,
}

impl OpEntry {
    pub const fn new(mnemonic: Mnemonic, addr_mode: AddrMode) -> Self {
        Self { mnemonic, addr_mode }
    }

    /// Number of bytes for this instruction (1, 2, or 3).
    pub const fn bytes(&self) -> u8 {
        match self.addr_mode {
            AddrMode::Implied | AddrMode::Accumulator => 1,
            AddrMode::Immediate | AddrMode::ZeroPage | AddrMode::ZeroPageX
            | AddrMode::ZeroPageY | AddrMode::IndirectX | AddrMode::IndirectY
            | AddrMode::Relative => 2,
            AddrMode::Absolute | AddrMode::AbsoluteX | AddrMode::AbsoluteY
            | AddrMode::Indirect => 3,
        }
    }
}

const ILL_ENTRY: OpEntry = OpEntry::new(Mnemonic::Ill, AddrMode::Implied);

pub const BRK_IRQ: u8 = 0x02;
pub const BRK_NMI: u8 = 0x04;
pub const BRK_RESET: u8 = 0x08;

pub struct Mos6502Input {
    pub data: u8,
    pub irq: bool,
    pub nmi: bool,
    pub ready: bool,
}

pub struct Mos6502Output {
    pub address: u16,
    pub data: u8,
    pub rw: bool,
    pub sync: bool,
}

pub struct Mos6502 {
    // --- Programmer-visible registers ---
    pub pc: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub p: u8,

    // --- Internal state ---
    tstate: u16,
    data_latch: u8,
    base_addr: u16,

    // --- Interrupt state ---
    brk_flags: u8,
    irq_shift: u8,     // IRQ pipeline shift register (bit 2 checked at fetch_opcode)
    nmi_shift: u8,     // NMI pipeline: bit 0 = pending latch, bit 2 checked at fetch_opcode
    nmi_prev: bool,    // previous NMI input level for edge detection
}

impl Mos6502 {
    pub fn new() -> Self {
        Self {
            pc: 0,
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFF,
            p: U | I,

            tstate: 0,
            brk_flags: BRK_RESET,
            data_latch: 0,
            base_addr: 0,

            irq_shift: 0,
            nmi_shift: 0,
            nmi_prev: false,
        }
    }

    #[inline(always)]
    pub fn phi1(&mut self) -> Mos6502Output {
        table::dispatch(self)
    }

    #[inline(always)]
    pub fn phi2(&mut self, input: &Mos6502Input) {
        self.data_latch = input.data;

        // IRQ pipeline: shift in (irq active AND interrupts enabled).
        self.irq_shift = (self.irq_shift << 1) | (input.irq && (self.p & I) == 0) as u8;

        // NMI pipeline: bit 0 is the pending latch, set on rising edge.
        // Shift upper bits, preserve bit 0 (sticky), OR in new edge.
        let edge = input.nmi && !self.nmi_prev;
        self.nmi_shift = (self.nmi_shift << 1) | (self.nmi_shift & 1) | edge as u8;
        self.nmi_prev = input.nmi;
    }

    // --- Helpers ---

    #[inline(always)]
    pub(crate) fn set_nz(&mut self, val: u8) {
        self.p = (self.p & !(N | Z)) | (val & N) | if val == 0 { Z } else { 0 };
    }

    #[inline(always)]
    pub(crate) fn set_flag(&mut self, flag: u8, set: bool) {
        if set {
            self.p |= flag;
        } else {
            self.p &= !flag;
        }
    }

    #[inline(always)]
    pub(crate) fn next_state(&mut self) {
        self.tstate += 1;
    }

    #[inline(always)]
    pub(crate) fn skip_next_state(&mut self) {
        self.tstate += 2;
    }

    #[inline(always)]
    pub(crate) fn inc_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    pub fn reset(&mut self) {
        self.tstate = 0;
        self.brk_flags = BRK_RESET;
        self.p |= I;
    }

    /// Start execution at the given PC. The caller must provide the opcode
    /// byte at that address. This simulates fetch_opcode having just run:
    /// it sets PC, feeds the opcode into the decode logic, and sets tstate
    /// so the next phi1 call executes the first micro-op of that instruction.
    pub fn set_pc(&mut self, pc: u16, opcode: u8) {
        self.pc = pc;
        self.data_latch = opcode;
        self.brk_flags = 0;
        addr::fetch_opcode(self);
    }
}

#[inline(always)]
pub(crate) fn read(address: u16) -> Mos6502Output {
    Mos6502Output { address, data: 0, rw: true, sync: false }
}

#[inline(always)]
pub(crate) fn sync_read(address: u16) -> Mos6502Output {
    Mos6502Output { address, data: 0, rw: true, sync: true }
}

#[inline(always)]
pub(crate) fn write(address: u16, data: u8) -> Mos6502Output {
    Mos6502Output { address, data, rw: false, sync: false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::table::DISASM;

    #[test]
    fn disasm_table_spot_check() {
        // Every legal opcode should have a non-Ill mnemonic.
        // Spot-check a representative sample across all addressing modes.
        let cases: &[(u8, Mnemonic, AddrMode)] = &[
            (0x00, Mnemonic::Brk, AddrMode::Implied),
            (0x69, Mnemonic::Adc, AddrMode::Immediate),
            (0xA5, Mnemonic::Lda, AddrMode::ZeroPage),
            (0x85, Mnemonic::Sta, AddrMode::ZeroPage),
            (0xB5, Mnemonic::Lda, AddrMode::ZeroPageX),
            (0xB6, Mnemonic::Ldx, AddrMode::ZeroPageY),
            (0x95, Mnemonic::Sta, AddrMode::ZeroPageX),
            (0x96, Mnemonic::Stx, AddrMode::ZeroPageY),
            (0xAD, Mnemonic::Lda, AddrMode::Absolute),
            (0x8D, Mnemonic::Sta, AddrMode::Absolute),
            (0xBD, Mnemonic::Lda, AddrMode::AbsoluteX),
            (0xB9, Mnemonic::Lda, AddrMode::AbsoluteY),
            (0x9D, Mnemonic::Sta, AddrMode::AbsoluteX),
            (0x99, Mnemonic::Sta, AddrMode::AbsoluteY),
            (0xA1, Mnemonic::Lda, AddrMode::IndirectX),
            (0x81, Mnemonic::Sta, AddrMode::IndirectX),
            (0xB1, Mnemonic::Lda, AddrMode::IndirectY),
            (0x91, Mnemonic::Sta, AddrMode::IndirectY),
            (0x0A, Mnemonic::Asl, AddrMode::Accumulator),
            (0x06, Mnemonic::Asl, AddrMode::ZeroPage),
            (0x0E, Mnemonic::Asl, AddrMode::Absolute),
            (0x1E, Mnemonic::Asl, AddrMode::AbsoluteX),
            (0xEA, Mnemonic::Nop, AddrMode::Implied),
            (0xE8, Mnemonic::Inx, AddrMode::Implied),
            (0x18, Mnemonic::Clc, AddrMode::Implied),
            (0x90, Mnemonic::Bcc, AddrMode::Relative),
            (0xF0, Mnemonic::Beq, AddrMode::Relative),
            (0x4C, Mnemonic::Jmp, AddrMode::Absolute),
            (0x6C, Mnemonic::Jmp, AddrMode::Indirect),
            (0x20, Mnemonic::Jsr, AddrMode::Absolute),
            (0x60, Mnemonic::Rts, AddrMode::Implied),
            (0x40, Mnemonic::Rti, AddrMode::Implied),
            (0x48, Mnemonic::Pha, AddrMode::Implied),
            (0x08, Mnemonic::Php, AddrMode::Implied),
            (0x68, Mnemonic::Pla, AddrMode::Implied),
            (0x28, Mnemonic::Plp, AddrMode::Implied),
        ];

        for &(opcode, mnemonic, addr_mode) in cases {
            let entry = DISASM[opcode as usize];
            assert_eq!(entry.mnemonic, mnemonic,
                "opcode ${:02X}: expected {:?}, got {:?}", opcode, mnemonic, entry.mnemonic);
            assert_eq!(entry.addr_mode, addr_mode,
                "opcode ${:02X}: expected {:?}, got {:?}", opcode, addr_mode, entry.addr_mode);
        }
    }

    #[test]
    fn disasm_table_illegal_opcodes_are_ill() {
        // A few known illegal opcodes should be Ill.
        for opcode in [0x02, 0x03, 0x04, 0x0B, 0x0C, 0x12, 0x22, 0x42, 0x62] {
            assert_eq!(DISASM[opcode].mnemonic, Mnemonic::Ill,
                "opcode ${:02X} should be Ill", opcode);
        }
    }

    #[test]
    fn disasm_table_byte_counts() {
        // Spot-check instruction byte counts.
        assert_eq!(DISASM[0xEA].bytes(), 1); // NOP — implied
        assert_eq!(DISASM[0xA9].bytes(), 2); // LDA #imm
        assert_eq!(DISASM[0xA5].bytes(), 2); // LDA zp
        assert_eq!(DISASM[0xAD].bytes(), 3); // LDA abs
        assert_eq!(DISASM[0x90].bytes(), 2); // BCC rel
        assert_eq!(DISASM[0x0A].bytes(), 1); // ASL A
        assert_eq!(DISASM[0x4C].bytes(), 3); // JMP abs
        assert_eq!(DISASM[0x6C].bytes(), 3); // JMP (ind)
    }
}
