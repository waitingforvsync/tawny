/// ALU and register operations.
///
/// Each operation reads from `cpu.data_latch` (for read-type ops) or
/// `cpu.a`/`cpu.x`/`cpu.y` (for stores) and updates registers/flags.
///
/// Operations are identified by `u8` constants so they can be used as
/// const generic parameters (Rust stable doesn't support const enum generics).
use super::flags::*;
use super::Mos6502;

// --- Operation identifiers (used as const generic parameters) ---

pub const ADC: u8 = 0;
pub const SBC: u8 = 1;
pub const AND: u8 = 2;
pub const ORA: u8 = 3;
pub const EOR: u8 = 4;
pub const CMP: u8 = 5;
pub const CPX: u8 = 6;
pub const CPY: u8 = 7;
pub const BIT: u8 = 8;
pub const LDA: u8 = 9;
pub const LDX: u8 = 10;
pub const LDY: u8 = 11;
pub const STA: u8 = 12;
pub const STX: u8 = 13;
pub const STY: u8 = 14;
pub const ASL: u8 = 15;
pub const LSR: u8 = 16;
pub const ROL: u8 = 17;
pub const ROR: u8 = 18;
pub const INC: u8 = 19;
pub const DEC: u8 = 20;
pub const INX: u8 = 21;
pub const INY: u8 = 22;
pub const DEX: u8 = 23;
pub const DEY: u8 = 24;
pub const TAX: u8 = 25;
pub const TAY: u8 = 26;
pub const TXA: u8 = 27;
pub const TYA: u8 = 28;
pub const TSX: u8 = 29;
pub const TXS: u8 = 30;
pub const CLC: u8 = 31;
pub const SEC: u8 = 32;
pub const CLI: u8 = 33;
pub const SEI: u8 = 34;
pub const CLV: u8 = 35;
pub const CLD: u8 = 36;
pub const SED: u8 = 37;
pub const NOP: u8 = 38;

// --- Read operations: use data_latch as input ---

/// Execute a read-type operation using `cpu.data_latch`.
/// Called by the final micro-op step of read instructions.
#[inline(always)]
pub fn execute_read<const OP: u8>(cpu: &mut Mos6502) {
    match OP {
        ADC => adc(cpu),
        SBC => sbc(cpu),
        AND => {
            cpu.a &= cpu.data_latch;
            cpu.set_nz(cpu.a);
        }
        ORA => {
            cpu.a |= cpu.data_latch;
            cpu.set_nz(cpu.a);
        }
        EOR => {
            cpu.a ^= cpu.data_latch;
            cpu.set_nz(cpu.a);
        }
        CMP => compare(cpu, cpu.a, cpu.data_latch),
        CPX => compare(cpu, cpu.x, cpu.data_latch),
        CPY => compare(cpu, cpu.y, cpu.data_latch),
        BIT => {
            cpu.set_flag(Z, (cpu.a & cpu.data_latch) == 0);
            cpu.set_flag(N, cpu.data_latch & N != 0);
            cpu.set_flag(V, cpu.data_latch & V != 0);
        }
        LDA => {
            cpu.a = cpu.data_latch;
            cpu.set_nz(cpu.a);
        }
        LDX => {
            cpu.x = cpu.data_latch;
            cpu.set_nz(cpu.x);
        }
        LDY => {
            cpu.y = cpu.data_latch;
            cpu.set_nz(cpu.y);
        }
        _ => {}
    }
}

// --- Read-modify-write operations: transform data_latch, return result ---

/// Execute an RMW operation on `cpu.data_latch`. Returns the modified value.
/// Called by the modify step of RMW instructions.
#[inline(always)]
pub fn execute_rmw<const OP: u8>(cpu: &mut Mos6502) -> u8 {
    match OP {
        ASL => {
            cpu.set_flag(C, cpu.data_latch & 0x80 != 0);
            let result = cpu.data_latch << 1;
            cpu.set_nz(result);
            result
        }
        LSR => {
            cpu.set_flag(C, cpu.data_latch & 0x01 != 0);
            let result = cpu.data_latch >> 1;
            cpu.set_nz(result);
            result
        }
        ROL => {
            let carry_in = cpu.p & C;
            cpu.set_flag(C, cpu.data_latch & 0x80 != 0);
            let result = (cpu.data_latch << 1) | carry_in;
            cpu.set_nz(result);
            result
        }
        ROR => {
            let carry_in = (cpu.p & C) << 7;
            cpu.set_flag(C, cpu.data_latch & 0x01 != 0);
            let result = (cpu.data_latch >> 1) | carry_in;
            cpu.set_nz(result);
            result
        }
        INC => {
            let result = cpu.data_latch.wrapping_add(1);
            cpu.set_nz(result);
            result
        }
        DEC => {
            let result = cpu.data_latch.wrapping_sub(1);
            cpu.set_nz(result);
            result
        }
        _ => cpu.data_latch,
    }
}

// --- Implied operations: operate on registers only ---

/// Execute an implied operation (no memory operand).
/// Called by the final micro-op step of implied instructions.
#[inline(always)]
pub fn execute_implied<const OP: u8>(cpu: &mut Mos6502) {
    match OP {
        INX => {
            cpu.x = cpu.x.wrapping_add(1);
            cpu.set_nz(cpu.x);
        }
        INY => {
            cpu.y = cpu.y.wrapping_add(1);
            cpu.set_nz(cpu.y);
        }
        DEX => {
            cpu.x = cpu.x.wrapping_sub(1);
            cpu.set_nz(cpu.x);
        }
        DEY => {
            cpu.y = cpu.y.wrapping_sub(1);
            cpu.set_nz(cpu.y);
        }
        TAX => {
            cpu.x = cpu.a;
            cpu.set_nz(cpu.x);
        }
        TAY => {
            cpu.y = cpu.a;
            cpu.set_nz(cpu.y);
        }
        TXA => {
            cpu.a = cpu.x;
            cpu.set_nz(cpu.a);
        }
        TYA => {
            cpu.a = cpu.y;
            cpu.set_nz(cpu.a);
        }
        TSX => {
            cpu.x = cpu.sp;
            cpu.set_nz(cpu.x);
        }
        TXS => {
            cpu.sp = cpu.x;
            // TXS does NOT set flags
        }
        CLC => cpu.p &= !C,
        SEC => cpu.p |= C,
        CLI => cpu.p &= !I,
        SEI => cpu.p |= I,
        CLV => cpu.p &= !V,
        CLD => cpu.p &= !D,
        SED => cpu.p |= D,
        NOP => {}
        _ => {}
    }
}

// --- Accumulator RMW: shift/rotate the accumulator directly ---

/// Execute an accumulator-mode RMW (ASL A, LSR A, ROL A, ROR A).
#[inline(always)]
pub fn execute_accumulator<const OP: u8>(cpu: &mut Mos6502) {
    // Temporarily put A into data_latch so we can reuse the RMW logic.
    cpu.data_latch = cpu.a;
    cpu.a = execute_rmw::<OP>(cpu);
}

// --- Store operations: return the value to write ---

/// Return the register value for a store instruction.
#[inline(always)]
pub fn store_value<const OP: u8>(cpu: &Mos6502) -> u8 {
    match OP {
        STA => cpu.a,
        STX => cpu.x,
        STY => cpu.y,
        _ => 0,
    }
}

// --- Internal helpers ---

fn adc(cpu: &mut Mos6502) {
    if cpu.p & D != 0 {
        adc_decimal(cpu);
    } else {
        adc_binary(cpu);
    }
}

fn adc_binary(cpu: &mut Mos6502) {
    let a = cpu.a as u16;
    let m = cpu.data_latch as u16;
    let c = (cpu.p & C) as u16;
    let sum = a + m + c;

    cpu.set_flag(C, sum > 0xFF);
    // Overflow: set if sign of result differs from sign of BOTH inputs
    cpu.set_flag(V, (!(a ^ m) & (a ^ sum) & 0x80) != 0);
    cpu.a = sum as u8;
    cpu.set_nz(cpu.a);
}

fn adc_decimal(cpu: &mut Mos6502) {
    let a = cpu.a as u16;
    let m = cpu.data_latch as u16;
    let c = (cpu.p & C) as u16;

    // Z flag is based on the binary result (NMOS quirk).
    let bin_sum = a + m + c;
    cpu.set_flag(Z, (bin_sum & 0xFF) == 0);

    // Low nybble with BCD correction.
    let mut lo = (a & 0x0F) + (m & 0x0F) + c;
    if lo > 0x09 {
        lo += 0x06;
    }

    // High nybble: sum with carry from low nybble.
    let mut sum = (a & 0xF0) + (m & 0xF0) + (if lo > 0x0F { 0x10 } else { 0 }) + (lo & 0x0F);

    // N and V are set from this intermediate result (after low correction,
    // before high correction) — NMOS behaviour.
    cpu.set_flag(N, (sum & 0x80) != 0);
    cpu.set_flag(V, (!(a ^ m) & (a ^ sum) & 0x80) != 0);

    // High nybble BCD correction.
    if sum > 0x9F {
        sum += 0x60;
    }
    cpu.set_flag(C, sum > 0xFF);

    cpu.a = sum as u8;
}

fn sbc(cpu: &mut Mos6502) {
    if cpu.p & D != 0 {
        sbc_decimal(cpu);
    } else {
        // SBC is equivalent to ADC with the operand complemented.
        let original = cpu.data_latch;
        cpu.data_latch = !cpu.data_latch;
        adc_binary(cpu);
        cpu.data_latch = original;
    }
}

fn sbc_decimal(cpu: &mut Mos6502) {
    let a = cpu.a as u16;
    let m = cpu.data_latch as u16;
    let c = (cpu.p & C) as u16;

    // On NMOS 6502, SBC decimal flags (N, V, Z, C) are all based on the
    // binary result — only the accumulator result is BCD-corrected.
    let bin = a.wrapping_sub(m).wrapping_sub(1 - c);
    cpu.set_flag(Z, (bin & 0xFF) == 0);
    cpu.set_flag(N, (bin & 0x80) != 0);
    cpu.set_flag(V, ((a ^ m) & (a ^ bin) & 0x80) != 0);
    cpu.set_flag(C, bin < 0x100);

    // BCD correction for the result.
    let lo = (a & 0x0F).wrapping_sub(m & 0x0F).wrapping_sub(1 - c);
    let mut result = if lo & 0x10 != 0 {
        ((lo.wrapping_sub(6)) & 0x0F) | ((a & 0xF0).wrapping_sub(m & 0xF0).wrapping_sub(0x10))
    } else {
        (lo & 0x0F) | ((a & 0xF0).wrapping_sub(m & 0xF0))
    };
    if result & 0x100 != 0 {
        result = result.wrapping_sub(0x60);
    }

    cpu.a = result as u8;
}

fn compare(cpu: &mut Mos6502, reg: u8, val: u8) {
    let result = reg.wrapping_sub(val);
    cpu.set_nz(result);
    cpu.set_flag(C, reg >= val);
}
