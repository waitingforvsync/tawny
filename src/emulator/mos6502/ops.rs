/// ALU and register operations.
///
/// Each operation is a zero-sized type implementing one of the operation
/// traits: `ReadOp`, `StoreOp`, `RmwOp`, `ImpliedOp`, `PushOp`, `PullOp`.
/// Micro-ops in `addr.rs` are generic over these traits, giving compile-time
/// enforcement that only valid ops are used with each addressing mode.
use super::flags::*;
use super::Mos6502;

// ======================================================================
// Operation traits
// ======================================================================

pub trait ReadOp {
    fn execute(cpu: &mut Mos6502, val: u8);
}

pub trait StoreOp {
    fn value(cpu: &Mos6502) -> u8;
}

pub trait RmwOp {
    fn execute(cpu: &mut Mos6502, val: u8) -> u8;
}

pub trait ImpliedOp {
    fn execute(cpu: &mut Mos6502);
}

pub trait PushOp {
    fn value(cpu: &Mos6502) -> u8;
}

pub trait PullOp {
    fn execute(cpu: &mut Mos6502, val: u8);
}

// ======================================================================
// Zero-sized operation types
// ======================================================================

pub struct Adc;
pub struct Sbc;
pub struct And;
pub struct Ora;
pub struct Eor;
pub struct Cmp;
pub struct Cpx;
pub struct Cpy;
pub struct Bit;
pub struct Lda;
pub struct Ldx;
pub struct Ldy;
pub struct Sta;
pub struct Stx;
pub struct Sty;
pub struct Asl;
pub struct Lsr;
pub struct Rol;
pub struct Ror;
pub struct Inc;
pub struct Dec;
pub struct Inx;
pub struct Iny;
pub struct Dex;
pub struct Dey;
pub struct Tax;
pub struct Tay;
pub struct Txa;
pub struct Tya;
pub struct Tsx;
pub struct Txs;
pub struct Clc;
pub struct Sec;
pub struct Cli;
pub struct Sei;
pub struct Clv;
pub struct Cld;
pub struct Sed;
pub struct Nop;
pub struct Pha;
pub struct Php;
pub struct Pla;
pub struct Plp;

// ======================================================================
// ReadOp implementations
// ======================================================================

impl ReadOp for Adc {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) { adc(cpu, val); }
}

impl ReadOp for Sbc {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) { sbc(cpu, val); }
}

impl ReadOp for And {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) {
        cpu.a &= val;
        cpu.set_nz(cpu.a);
    }
}

impl ReadOp for Ora {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) {
        cpu.a |= val;
        cpu.set_nz(cpu.a);
    }
}

impl ReadOp for Eor {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) {
        cpu.a ^= val;
        cpu.set_nz(cpu.a);
    }
}

impl ReadOp for Cmp {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) { compare(cpu, cpu.a, val); }
}

impl ReadOp for Cpx {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) { compare(cpu, cpu.x, val); }
}

impl ReadOp for Cpy {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) { compare(cpu, cpu.y, val); }
}

impl ReadOp for Bit {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) {
        cpu.set_flag(Z, (cpu.a & val) == 0);
        cpu.set_flag(N, val & N != 0);
        cpu.set_flag(V, val & V != 0);
    }
}

impl ReadOp for Lda {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) {
        cpu.a = val;
        cpu.set_nz(cpu.a);
    }
}

impl ReadOp for Ldx {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) {
        cpu.x = val;
        cpu.set_nz(cpu.x);
    }
}

impl ReadOp for Ldy {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) {
        cpu.y = val;
        cpu.set_nz(cpu.y);
    }
}

// ======================================================================
// StoreOp implementations
// ======================================================================

impl StoreOp for Sta {
    #[inline(always)]
    fn value(cpu: &Mos6502) -> u8 { cpu.a }
}

impl StoreOp for Stx {
    #[inline(always)]
    fn value(cpu: &Mos6502) -> u8 { cpu.x }
}

impl StoreOp for Sty {
    #[inline(always)]
    fn value(cpu: &Mos6502) -> u8 { cpu.y }
}

// ======================================================================
// RmwOp implementations
// ======================================================================

impl RmwOp for Asl {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) -> u8 {
        cpu.set_flag(C, val & 0x80 != 0);
        let result = val << 1;
        cpu.set_nz(result);
        result
    }
}

impl RmwOp for Lsr {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) -> u8 {
        cpu.set_flag(C, val & 0x01 != 0);
        let result = val >> 1;
        cpu.set_nz(result);
        result
    }
}

impl RmwOp for Rol {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) -> u8 {
        let carry_in = cpu.p & C;
        cpu.set_flag(C, val & 0x80 != 0);
        let result = (val << 1) | carry_in;
        cpu.set_nz(result);
        result
    }
}

impl RmwOp for Ror {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) -> u8 {
        let carry_in = (cpu.p & C) << 7;
        cpu.set_flag(C, val & 0x01 != 0);
        let result = (val >> 1) | carry_in;
        cpu.set_nz(result);
        result
    }
}

impl RmwOp for Inc {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) -> u8 {
        let result = val.wrapping_add(1);
        cpu.set_nz(result);
        result
    }
}

impl RmwOp for Dec {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) -> u8 {
        let result = val.wrapping_sub(1);
        cpu.set_nz(result);
        result
    }
}

// ======================================================================
// ImpliedOp implementations
// ======================================================================

impl ImpliedOp for Inx {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) {
        cpu.x = cpu.x.wrapping_add(1);
        cpu.set_nz(cpu.x);
    }
}

impl ImpliedOp for Iny {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) {
        cpu.y = cpu.y.wrapping_add(1);
        cpu.set_nz(cpu.y);
    }
}

impl ImpliedOp for Dex {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) {
        cpu.x = cpu.x.wrapping_sub(1);
        cpu.set_nz(cpu.x);
    }
}

impl ImpliedOp for Dey {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) {
        cpu.y = cpu.y.wrapping_sub(1);
        cpu.set_nz(cpu.y);
    }
}

impl ImpliedOp for Tax {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) {
        cpu.x = cpu.a;
        cpu.set_nz(cpu.x);
    }
}

impl ImpliedOp for Tay {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) {
        cpu.y = cpu.a;
        cpu.set_nz(cpu.y);
    }
}

impl ImpliedOp for Txa {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) {
        cpu.a = cpu.x;
        cpu.set_nz(cpu.a);
    }
}

impl ImpliedOp for Tya {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) {
        cpu.a = cpu.y;
        cpu.set_nz(cpu.a);
    }
}

impl ImpliedOp for Tsx {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) {
        cpu.x = cpu.sp;
        cpu.set_nz(cpu.x);
    }
}

impl ImpliedOp for Txs {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) { cpu.sp = cpu.x; }
}

impl ImpliedOp for Clc {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) { cpu.p &= !C; }
}

impl ImpliedOp for Sec {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) { cpu.p |= C; }
}

impl ImpliedOp for Cli {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) { cpu.p &= !I; }
}

impl ImpliedOp for Sei {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) { cpu.p |= I; }
}

impl ImpliedOp for Clv {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) { cpu.p &= !V; }
}

impl ImpliedOp for Cld {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) { cpu.p &= !D; }
}

impl ImpliedOp for Sed {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) { cpu.p |= D; }
}

impl ImpliedOp for Nop {
    #[inline(always)]
    fn execute(_cpu: &mut Mos6502) {}
}

// ======================================================================
// PushOp implementations
// ======================================================================

impl PushOp for Pha {
    #[inline(always)]
    fn value(cpu: &Mos6502) -> u8 { cpu.a }
}

impl PushOp for Php {
    #[inline(always)]
    fn value(cpu: &Mos6502) -> u8 { cpu.p | B | U }
}

// ======================================================================
// PullOp implementations
// ======================================================================

impl PullOp for Pla {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) {
        cpu.a = val;
        cpu.set_nz(cpu.a);
    }
}

impl PullOp for Plp {
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) {
        cpu.p = (val & !(B | U)) | U;
    }
}

// ======================================================================
// Internal helpers
// ======================================================================

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
