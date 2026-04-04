pub mod addr;
pub mod flags;
pub mod ops;
pub mod table;

use flags::*;

/// Table indexed by (opcode << 3) | step. 256 opcodes * 8 steps max.
pub const MAX_STEPS: usize = 8;
pub const TABLE_SIZE: usize = 256 * MAX_STEPS;

/// A micro-op: runs during phi1, returns output pins for the bus cycle.
pub type MicroOp = fn(&mut Mos6502) -> Mos6502Output;

// --- BRK flags: what caused the current BRK sequence ---
pub const BRK_SOFTWARE: u8 = 0x01;
pub const BRK_IRQ: u8 = 0x02;
pub const BRK_NMI: u8 = 0x04;
pub const BRK_RESET: u8 = 0x08;

/// MOS 6502 input pins (active during phi2).
pub struct Mos6502Input {
    pub data: u8,
    pub irq: bool,
    pub nmi: bool,
    pub ready: bool,
}

/// MOS 6502 output pins (driven during phi1).
pub struct Mos6502Output {
    pub address: u16,
    pub data: u8,
    pub rw: bool,
    pub sync: bool,
}

/// MOS 6502 CPU.
pub struct Mos6502 {
    // --- Programmer-visible registers ---
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    /// Processor status: NV-BDIZC
    pub p: u8,

    // --- Internal state ---
    /// Index into the STEPS table: (opcode << 3) | step_within_instruction.
    tstate: usize,
    /// Last value latched from the data bus (set by phi2).
    data_latch: u8,
    /// Address accumulator for multi-byte addressing modes.
    base_addr: u16,
    /// Page-cross flag for indexed addressing.
    page_crossed: bool,
    /// Result of an RMW modify step (survives phi2, unlike data_latch).
    rmw_result: u8,
    /// Set by setup_opcode_fetch; phi2 checks this to know when to decode.
    sync: bool,

    // --- Interrupt/BRK state ---
    /// What caused the current BRK sequence (0 until BRK T0 sets it).
    brk_flags: u8,
    irq_latch: bool,
    nmi_latch: bool,
    /// NMI is edge-triggered: set on rising edge, cleared when serviced.
    nmi_pending: bool,
}

impl Mos6502 {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD,
            pc: 0,
            p: U | I,

            tstate: 0,
            brk_flags: BRK_RESET,
            data_latch: 0,
            base_addr: 0,
            page_crossed: false,
            rmw_result: 0,
            sync: false,

            irq_latch: false,
            nmi_latch: false,
            nmi_pending: false,
        }
    }

    /// Phi1: advance the state machine. Returns output pins for bus access.
    #[inline(always)]
    pub fn phi1(&mut self) -> Mos6502Output {
        self.sync = false;
        table::dispatch(self)
    }

    /// Phi2: latch data bus, decode opcode if sync, shift interrupt pipeline.
    #[inline(always)]
    pub fn phi2(&mut self, input: &Mos6502Input) {
        self.data_latch = input.data;

        if self.sync {
            self.tstate = (self.data_latch as usize) << 3;
        }

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
    pub(crate) fn next(&mut self) {
        self.tstate += 1;
    }

    /// Opcode fetch: checks interrupts, returns output for the fetch cycle.
    /// If interrupt pending: forces BRK, sync stays false.
    /// Otherwise: fetches from PC, sets sync for phi2 decode.
    pub(crate) fn setup_opcode_fetch(&mut self) -> Mos6502Output {
        if self.nmi_pending {
            self.nmi_pending = false;
            self.brk_flags = BRK_NMI;
            self.tstate = 0;
            Mos6502Output { address: self.pc, data: 0, rw: true, sync: false }
        } else if self.irq_latch && (self.p & I) == 0 {
            self.brk_flags = BRK_IRQ;
            self.tstate = 0;
            Mos6502Output { address: self.pc, data: 0, rw: true, sync: false }
        } else {
            self.brk_flags = 0;
            let addr = self.pc;
            self.pc = self.pc.wrapping_add(1);
            self.sync = true;
            Mos6502Output { address: addr, data: 0, rw: true, sync: true }
        }
    }

    pub fn reset(&mut self) {
        self.tstate = 0;
        self.brk_flags = BRK_RESET;
        self.p |= I;
        self.sync = false;
    }
}

/// Helper to build a read output.
#[inline(always)]
pub(crate) fn read(address: u16) -> Mos6502Output {
    Mos6502Output { address, data: 0, rw: true, sync: false }
}

/// Helper to build a write output.
#[inline(always)]
pub(crate) fn write(address: u16, data: u8) -> Mos6502Output {
    Mos6502Output { address, data, rw: false, sync: false }
}
