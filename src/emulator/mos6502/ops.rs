/// ALU and register operations.
///
/// Each operation is a zero-sized type implementing one of the operation
/// traits: `ReadOp`, `StoreOp`, `RmwOp`, `ImpliedOp`, `PushOp`, `PullOp`.
/// Micro-ops in `addr.rs` are generic over these traits, giving compile-time
/// enforcement that only valid ops are used with each addressing mode.
use super::flags::*;
use super::Mnemonic;
use super::Mnemonic as M;
use super::Mos6502;

// ======================================================================
// Operation traits
// ======================================================================

pub trait ReadOp {
    const MNEMONIC: Mnemonic;
    fn execute(cpu: &mut Mos6502, val: u8);
}

pub trait StoreOp {
    const MNEMONIC: Mnemonic;
    fn value(cpu: &Mos6502) -> u8;
}

pub trait RmwOp {
    const MNEMONIC: Mnemonic;
    fn execute(cpu: &mut Mos6502, val: u8) -> u8;
}

pub trait ImpliedOp {
    const MNEMONIC: Mnemonic;
    fn execute(cpu: &mut Mos6502);
}

pub trait PushOp {
    const MNEMONIC: Mnemonic;
    fn value(cpu: &Mos6502) -> u8;
}

pub trait PullOp {
    const MNEMONIC: Mnemonic;
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
    const MNEMONIC: Mnemonic = M::Adc;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) { adc(cpu, val); }
}

impl ReadOp for Sbc {
    const MNEMONIC: Mnemonic = M::Sbc;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) { sbc(cpu, val); }
}

impl ReadOp for And {
    const MNEMONIC: Mnemonic = M::And;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) {
        cpu.a &= val;
        cpu.set_nz(cpu.a);
    }
}

impl ReadOp for Ora {
    const MNEMONIC: Mnemonic = M::Ora;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) {
        cpu.a |= val;
        cpu.set_nz(cpu.a);
    }
}

impl ReadOp for Eor {
    const MNEMONIC: Mnemonic = M::Eor;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) {
        cpu.a ^= val;
        cpu.set_nz(cpu.a);
    }
}

impl ReadOp for Cmp {
    const MNEMONIC: Mnemonic = M::Cmp;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) { compare(cpu, cpu.a, val); }
}

impl ReadOp for Cpx {
    const MNEMONIC: Mnemonic = M::Cpx;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) { compare(cpu, cpu.x, val); }
}

impl ReadOp for Cpy {
    const MNEMONIC: Mnemonic = M::Cpy;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) { compare(cpu, cpu.y, val); }
}

impl ReadOp for Bit {
    const MNEMONIC: Mnemonic = M::Bit;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) {
        cpu.set_flag(Z, (cpu.a & val) == 0);
        cpu.set_flag(N, val & N != 0);
        cpu.set_flag(V, val & V != 0);
    }
}

impl ReadOp for Lda {
    const MNEMONIC: Mnemonic = M::Lda;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) {
        cpu.a = val;
        cpu.set_nz(cpu.a);
    }
}

impl ReadOp for Ldx {
    const MNEMONIC: Mnemonic = M::Ldx;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) {
        cpu.x = val;
        cpu.set_nz(cpu.x);
    }
}

impl ReadOp for Ldy {
    const MNEMONIC: Mnemonic = M::Ldy;
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
    const MNEMONIC: Mnemonic = M::Sta;
    #[inline(always)]
    fn value(cpu: &Mos6502) -> u8 { cpu.a }
}

impl StoreOp for Stx {
    const MNEMONIC: Mnemonic = M::Stx;
    #[inline(always)]
    fn value(cpu: &Mos6502) -> u8 { cpu.x }
}

impl StoreOp for Sty {
    const MNEMONIC: Mnemonic = M::Sty;
    #[inline(always)]
    fn value(cpu: &Mos6502) -> u8 { cpu.y }
}

// ======================================================================
// RmwOp implementations
// ======================================================================

impl RmwOp for Asl {
    const MNEMONIC: Mnemonic = M::Asl;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) -> u8 {
        cpu.set_flag(C, val & 0x80 != 0);
        let result = val << 1;
        cpu.set_nz(result);
        result
    }
}

impl RmwOp for Lsr {
    const MNEMONIC: Mnemonic = M::Lsr;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) -> u8 {
        cpu.set_flag(C, val & 0x01 != 0);
        let result = val >> 1;
        cpu.set_nz(result);
        result
    }
}

impl RmwOp for Rol {
    const MNEMONIC: Mnemonic = M::Rol;
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
    const MNEMONIC: Mnemonic = M::Ror;
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
    const MNEMONIC: Mnemonic = M::Inc;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) -> u8 {
        let result = val.wrapping_add(1);
        cpu.set_nz(result);
        result
    }
}

impl RmwOp for Dec {
    const MNEMONIC: Mnemonic = M::Dec;
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
    const MNEMONIC: Mnemonic = M::Inx;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) {
        cpu.x = cpu.x.wrapping_add(1);
        cpu.set_nz(cpu.x);
    }
}

impl ImpliedOp for Iny {
    const MNEMONIC: Mnemonic = M::Iny;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) {
        cpu.y = cpu.y.wrapping_add(1);
        cpu.set_nz(cpu.y);
    }
}

impl ImpliedOp for Dex {
    const MNEMONIC: Mnemonic = M::Dex;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) {
        cpu.x = cpu.x.wrapping_sub(1);
        cpu.set_nz(cpu.x);
    }
}

impl ImpliedOp for Dey {
    const MNEMONIC: Mnemonic = M::Dey;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) {
        cpu.y = cpu.y.wrapping_sub(1);
        cpu.set_nz(cpu.y);
    }
}

impl ImpliedOp for Tax {
    const MNEMONIC: Mnemonic = M::Tax;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) {
        cpu.x = cpu.a;
        cpu.set_nz(cpu.x);
    }
}

impl ImpliedOp for Tay {
    const MNEMONIC: Mnemonic = M::Tay;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) {
        cpu.y = cpu.a;
        cpu.set_nz(cpu.y);
    }
}

impl ImpliedOp for Txa {
    const MNEMONIC: Mnemonic = M::Txa;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) {
        cpu.a = cpu.x;
        cpu.set_nz(cpu.a);
    }
}

impl ImpliedOp for Tya {
    const MNEMONIC: Mnemonic = M::Tya;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) {
        cpu.a = cpu.y;
        cpu.set_nz(cpu.a);
    }
}

impl ImpliedOp for Tsx {
    const MNEMONIC: Mnemonic = M::Tsx;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) {
        cpu.x = cpu.sp;
        cpu.set_nz(cpu.x);
    }
}

impl ImpliedOp for Txs {
    const MNEMONIC: Mnemonic = M::Txs;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) { cpu.sp = cpu.x; }
}

impl ImpliedOp for Clc {
    const MNEMONIC: Mnemonic = M::Clc;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) { cpu.p &= !C; }
}

impl ImpliedOp for Sec {
    const MNEMONIC: Mnemonic = M::Sec;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) { cpu.p |= C; }
}

impl ImpliedOp for Cli {
    const MNEMONIC: Mnemonic = M::Cli;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) { cpu.p &= !I; }
}

impl ImpliedOp for Sei {
    const MNEMONIC: Mnemonic = M::Sei;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) { cpu.p |= I; }
}

impl ImpliedOp for Clv {
    const MNEMONIC: Mnemonic = M::Clv;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) { cpu.p &= !V; }
}

impl ImpliedOp for Cld {
    const MNEMONIC: Mnemonic = M::Cld;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) { cpu.p &= !D; }
}

impl ImpliedOp for Sed {
    const MNEMONIC: Mnemonic = M::Sed;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502) { cpu.p |= D; }
}

impl ImpliedOp for Nop {
    const MNEMONIC: Mnemonic = M::Nop;
    #[inline(always)]
    fn execute(_cpu: &mut Mos6502) {}
}

// ======================================================================
// PushOp implementations
// ======================================================================

impl PushOp for Pha {
    const MNEMONIC: Mnemonic = M::Pha;
    #[inline(always)]
    fn value(cpu: &Mos6502) -> u8 { cpu.a }
}

impl PushOp for Php {
    const MNEMONIC: Mnemonic = M::Php;
    #[inline(always)]
    fn value(cpu: &Mos6502) -> u8 { cpu.p | B | U }
}

// ======================================================================
// PullOp implementations
// ======================================================================

impl PullOp for Pla {
    const MNEMONIC: Mnemonic = M::Pla;
    #[inline(always)]
    fn execute(cpu: &mut Mos6502, val: u8) {
        cpu.a = val;
        cpu.set_nz(cpu.a);
    }
}

impl PullOp for Plp {
    const MNEMONIC: Mnemonic = M::Plp;
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
