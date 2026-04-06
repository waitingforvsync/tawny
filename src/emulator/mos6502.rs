pub mod addr;
pub mod flags;
pub mod ops;
pub mod table;

use flags::*;

pub const MAX_STEPS: usize = 8;
pub const TABLE_SIZE: usize = 256 * MAX_STEPS;

pub type MicroOp = fn(&mut Mos6502) -> Mos6502Output;

pub const BRK_SOFTWARE: u8 = 0x01;
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
    rmw_result: u8,

    // --- Interrupt/BRK state ---
    brk_flags: u8,
    irq_latch: bool,
    nmi_latch: bool,
    nmi_pending: bool,
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
            rmw_result: 0,

            irq_latch: false,
            nmi_latch: false,
            nmi_pending: false,
        }
    }

    #[inline(always)]
    pub fn phi1(&mut self) -> Mos6502Output {
        table::dispatch(self)
    }

    #[inline(always)]
    pub fn phi2(&mut self, input: &Mos6502Input) {
        self.data_latch = input.data;
        self.irq_latch = input.irq;
        if input.nmi && !self.nmi_latch {
            self.nmi_pending = true;
        }
        self.nmi_latch = input.nmi;
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
