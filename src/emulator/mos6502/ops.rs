/// ALU and register operations.
///
/// Operations are identified by `u8` constants so they can be used as
/// const generic parameters.
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

// --- Read operations ---

/// Execute a read-type operation with the given value.
#[inline(always)]
pub fn execute_read<const OP: u8>(cpu: &mut Mos6502, val: u8) {
    match OP {
        ADC => adc(cpu, val),
        SBC => sbc(cpu, val),
        AND => {
            cpu.a &= val;
            cpu.set_nz(cpu.a);
        }
        ORA => {
            cpu.a |= val;
            cpu.set_nz(cpu.a);
        }
        EOR => {
            cpu.a ^= val;
            cpu.set_nz(cpu.a);
        }
        CMP => compare(cpu, cpu.a, val),
        CPX => compare(cpu, cpu.x, val),
        CPY => compare(cpu, cpu.y, val),
        BIT => {
            cpu.set_flag(Z, (cpu.a & val) == 0);
            cpu.set_flag(N, val & N != 0);
            cpu.set_flag(V, val & V != 0);
        }
        LDA => {
            cpu.a = val;
            cpu.set_nz(cpu.a);
        }
        LDX => {
            cpu.x = val;
            cpu.set_nz(cpu.x);
        }
        LDY => {
            cpu.y = val;
            cpu.set_nz(cpu.y);
        }
        _ => {}
    }
}

// --- Read-modify-write operations ---

/// Execute an RMW operation on `val`. Returns the modified value.
#[inline(always)]
pub fn execute_rmw<const OP: u8>(cpu: &mut Mos6502, val: u8) -> u8 {
    match OP {
        ASL => {
            cpu.set_flag(C, val & 0x80 != 0);
            let result = val << 1;
            cpu.set_nz(result);
            result
        }
        LSR => {
            cpu.set_flag(C, val & 0x01 != 0);
            let result = val >> 1;
            cpu.set_nz(result);
            result
        }
        ROL => {
            let carry_in = cpu.p & C;
            cpu.set_flag(C, val & 0x80 != 0);
            let result = (val << 1) | carry_in;
            cpu.set_nz(result);
            result
        }
        ROR => {
            let carry_in = (cpu.p & C) << 7;
            cpu.set_flag(C, val & 0x01 != 0);
            let result = (val >> 1) | carry_in;
            cpu.set_nz(result);
            result
        }
        INC => {
            let result = val.wrapping_add(1);
            cpu.set_nz(result);
            result
        }
        DEC => {
            let result = val.wrapping_sub(1);
            cpu.set_nz(result);
            result
        }
        _ => val,
    }
}

// --- Implied operations ---

/// Execute an implied operation (no memory operand).
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

// --- Accumulator RMW ---

/// Execute an accumulator-mode RMW (ASL A, LSR A, ROL A, ROR A).
#[inline(always)]
pub fn execute_accumulator<const OP: u8>(cpu: &mut Mos6502) {
    cpu.a = execute_rmw::<OP>(cpu, cpu.a);
}

// --- Store operations ---

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

fn adc(cpu: &mut Mos6502, val: u8) {
    if cpu.p & D != 0 {
        adc_decimal(cpu, val);
    } else {
        adc_binary(cpu, val);
    }
}

fn adc_binary(cpu: &mut Mos6502, val: u8) {
    let a = cpu.a as u16;
    let m = val as u16;
    let c = (cpu.p & C) as u16;
    let sum = a + m + c;

    cpu.set_flag(C, sum > 0xFF);
    cpu.set_flag(V, (!(a ^ m) & (a ^ sum) & 0x80) != 0);
    cpu.a = sum as u8;
    cpu.set_nz(cpu.a);
}

fn adc_decimal(cpu: &mut Mos6502, val: u8) {
    let a = cpu.a as u16;
    let m = val as u16;
    let c = (cpu.p & C) as u16;

    let bin_sum = a + m + c;
    cpu.set_flag(Z, (bin_sum & 0xFF) == 0);

    let mut lo = (a & 0x0F) + (m & 0x0F) + c;
    if lo > 0x09 {
        lo += 0x06;
    }

    let mut sum = (a & 0xF0) + (m & 0xF0) + (if lo > 0x0F { 0x10 } else { 0 }) + (lo & 0x0F);

    cpu.set_flag(N, (sum & 0x80) != 0);
    cpu.set_flag(V, (!(a ^ m) & (a ^ sum) & 0x80) != 0);

    if sum > 0x9F {
        sum += 0x60;
    }
    cpu.set_flag(C, sum > 0xFF);

    cpu.a = sum as u8;
}

fn sbc(cpu: &mut Mos6502, val: u8) {
    if cpu.p & D != 0 {
        sbc_decimal(cpu, val);
    } else {
        adc_binary(cpu, !val);
    }
}

fn sbc_decimal(cpu: &mut Mos6502, val: u8) {
    let a = cpu.a as u16;
    let m = val as u16;
    let c = (cpu.p & C) as u16;

    let bin = a.wrapping_sub(m).wrapping_sub(1 - c);
    cpu.set_flag(Z, (bin & 0xFF) == 0);
    cpu.set_flag(N, (bin & 0x80) != 0);
    cpu.set_flag(V, ((a ^ m) & (a ^ bin) & 0x80) != 0);
    cpu.set_flag(C, bin < 0x100);

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
