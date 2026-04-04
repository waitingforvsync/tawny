/// Addressing mode micro-op functions.
///
/// Each function runs during phi1, returning output pins for that bus cycle.
/// Every micro-op MUST explicitly advance tstate (usually via `cpu.next()`).
///
/// Naming convention:
/// - `addr_*`  — puts an address on the bus for a read
/// - `write_*` — puts an address + data on the bus for a write
/// - `fetch_*` — reads from PC and increments PC

use super::flags::*;
use super::ops;
use super::{read, write, Mos6502, Mos6502Output, BRK_NMI, BRK_RESET, BRK_SOFTWARE};

// ==========================================================================
// Opcode fetch (last step of every instruction)
// ==========================================================================

pub fn fetch_opcode(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.setup_opcode_fetch()
}

// ==========================================================================
// Common: fetch byte from PC
// ==========================================================================

pub fn fetch_operand(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = cpu.pc;
    cpu.pc = cpu.pc.wrapping_add(1);
    cpu.next();
    read(addr)
}

// ==========================================================================
// Final step for read instructions: operation + opcode fetch
// ==========================================================================

pub fn final_read<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    ops::execute_read::<OP>(cpu);
    cpu.setup_opcode_fetch()
}

// ==========================================================================
// Implied / Accumulator
// ==========================================================================

pub fn implied<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    ops::execute_implied::<OP>(cpu);
    cpu.next();
    read(cpu.pc)
}

pub fn accumulator<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    ops::execute_accumulator::<OP>(cpu);
    cpu.next();
    read(cpu.pc)
}

// ==========================================================================
// Zero page
// ==========================================================================

pub fn addr_zp(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16;
    cpu.next();
    read(cpu.base_addr)
}

pub fn write_zp<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = cpu.data_latch as u16;
    cpu.next();
    write(addr, ops::store_value::<OP>(cpu))
}

pub fn addr_zp_x(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = cpu.data_latch as u16;
    cpu.base_addr = cpu.data_latch.wrapping_add(cpu.x) as u16;
    cpu.next();
    read(addr)
}

pub fn addr_zp_y(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = cpu.data_latch as u16;
    cpu.base_addr = cpu.data_latch.wrapping_add(cpu.y) as u16;
    cpu.next();
    read(addr)
}

pub fn addr_zp_indexed(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next();
    read(cpu.base_addr)
}

pub fn write_zp_indexed<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next();
    write(cpu.base_addr, ops::store_value::<OP>(cpu))
}

// ==========================================================================
// Absolute
// ==========================================================================

pub fn fetch_addr_hi(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16;
    let addr = cpu.pc;
    cpu.pc = cpu.pc.wrapping_add(1);
    cpu.next();
    read(addr)
}

pub fn addr_abs(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr |= (cpu.data_latch as u16) << 8;
    cpu.next();
    read(cpu.base_addr)
}

pub fn write_abs<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr |= (cpu.data_latch as u16) << 8;
    cpu.next();
    write(cpu.base_addr, ops::store_value::<OP>(cpu))
}

pub fn fetch_addr_hi_add_x(cpu: &mut Mos6502) -> Mos6502Output {
    let lo = cpu.data_latch.wrapping_add(cpu.x);
    cpu.page_crossed = lo < cpu.data_latch;
    cpu.base_addr = lo as u16;
    let addr = cpu.pc;
    cpu.pc = cpu.pc.wrapping_add(1);
    cpu.next();
    read(addr)
}

pub fn fetch_addr_hi_add_y(cpu: &mut Mos6502) -> Mos6502Output {
    let lo = cpu.data_latch.wrapping_add(cpu.y);
    cpu.page_crossed = lo < cpu.data_latch;
    cpu.base_addr = lo as u16;
    let addr = cpu.pc;
    cpu.pc = cpu.pc.wrapping_add(1);
    cpu.next();
    read(addr)
}

pub fn addr_abs_indexed(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr |= (cpu.data_latch as u16) << 8;
    if cpu.page_crossed {
        cpu.next();
    } else {
        cpu.tstate += 2;
    }
    read(cpu.base_addr)
}

pub fn addr_abs_indexed_penalty(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr |= (cpu.data_latch as u16) << 8;
    cpu.next();
    read(cpu.base_addr)
}

pub fn addr_fixup(cpu: &mut Mos6502) -> Mos6502Output {
    if cpu.page_crossed {
        cpu.base_addr = cpu.base_addr.wrapping_add(0x100);
    }
    cpu.next();
    read(cpu.base_addr)
}

pub fn write_fixup<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    if cpu.page_crossed {
        cpu.base_addr = cpu.base_addr.wrapping_add(0x100);
    }
    cpu.next();
    write(cpu.base_addr, ops::store_value::<OP>(cpu))
}

// ==========================================================================
// (Indirect,X)
// ==========================================================================

pub fn addr_indirect_lo(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next();
    read(cpu.base_addr)
}

pub fn addr_indirect_hi(cpu: &mut Mos6502) -> Mos6502Output {
    let ptr_lo = cpu.base_addr;
    cpu.base_addr = cpu.data_latch as u16;
    cpu.next();
    read((ptr_lo.wrapping_add(1)) & 0x00FF)
}

pub fn addr_indirect_target(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr |= (cpu.data_latch as u16) << 8;
    cpu.next();
    read(cpu.base_addr)
}

pub fn write_indirect_target<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr |= (cpu.data_latch as u16) << 8;
    cpu.next();
    write(cpu.base_addr, ops::store_value::<OP>(cpu))
}

// ==========================================================================
// (Indirect),Y
// ==========================================================================

pub fn addr_ind_y_lo(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16;
    cpu.next();
    read(cpu.base_addr)
}

pub fn addr_ind_y_hi(cpu: &mut Mos6502) -> Mos6502Output {
    let lo = cpu.data_latch.wrapping_add(cpu.y);
    cpu.page_crossed = lo < cpu.data_latch;
    let ptr_lo = cpu.base_addr;
    cpu.base_addr = lo as u16;
    cpu.next();
    read((ptr_lo.wrapping_add(1)) & 0x00FF)
}

// ==========================================================================
// Read-modify-write
// ==========================================================================

pub fn rmw_modify<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    let original = cpu.data_latch;
    cpu.rmw_result = ops::execute_rmw::<OP>(cpu);
    cpu.next();
    write(cpu.base_addr, original)
}

pub fn rmw_write(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next();
    write(cpu.base_addr, cpu.rmw_result)
}

// ==========================================================================
// Branches
// ==========================================================================

pub fn branch<const FLAG: u8, const SET: bool>(cpu: &mut Mos6502) -> Mos6502Output {
    let taken = if SET { cpu.p & FLAG != 0 } else { cpu.p & FLAG == 0 };
    if taken {
        cpu.base_addr = cpu.data_latch as u16;
        cpu.next();
        read(cpu.pc)
    } else {
        cpu.setup_opcode_fetch()
    }
}

pub fn branch_take(cpu: &mut Mos6502) -> Mos6502Output {
    let offset = cpu.base_addr as u8 as i8;
    let old_pc = cpu.pc;
    cpu.pc = cpu.pc.wrapping_add(offset as i16 as u16);
    if (old_pc ^ cpu.pc) & 0xFF00 != 0 {
        cpu.next();
        read((old_pc & 0xFF00) | (cpu.pc & 0x00FF))
    } else {
        cpu.setup_opcode_fetch()
    }
}

pub fn branch_fixup(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.setup_opcode_fetch()
}

// ==========================================================================
// JMP
// ==========================================================================

pub fn jmp_abs(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.pc = cpu.base_addr | ((cpu.data_latch as u16) << 8);
    cpu.setup_opcode_fetch()
}

pub fn addr_jmp_indirect(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr |= (cpu.data_latch as u16) << 8;
    cpu.next();
    read(cpu.base_addr)
}

pub fn addr_jmp_indirect_hi(cpu: &mut Mos6502) -> Mos6502Output {
    let target_lo = cpu.data_latch;
    let hi_addr = (cpu.base_addr & 0xFF00) | ((cpu.base_addr.wrapping_add(1)) & 0x00FF);
    cpu.base_addr = target_lo as u16;
    cpu.next();
    read(hi_addr)
}

pub fn jmp_indirect_done(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.pc = cpu.base_addr | ((cpu.data_latch as u16) << 8);
    cpu.setup_opcode_fetch()
}

// ==========================================================================
// JSR
// ==========================================================================

pub fn jsr_save_lo(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16;
    cpu.next();
    read(0x0100 | cpu.sp as u16)
}

pub fn jsr_push_pch(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    let data = (cpu.pc >> 8) as u8;
    cpu.sp = cpu.sp.wrapping_sub(1);
    cpu.next();
    write(addr, data)
}

pub fn jsr_push_pcl(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    let data = cpu.pc as u8;
    cpu.sp = cpu.sp.wrapping_sub(1);
    cpu.next();
    write(addr, data)
}

pub fn jsr_fetch_hi(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next();
    read(cpu.pc)
}

pub fn jsr_done(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.pc = cpu.base_addr | ((cpu.data_latch as u16) << 8);
    cpu.setup_opcode_fetch()
}

// ==========================================================================
// RTS
// ==========================================================================

pub fn rts_dummy(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next();
    read(cpu.pc)
}

pub fn rts_inc_sp(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_add(1);
    cpu.next();
    read(addr)
}

pub fn rts_addr_pcl(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_add(1);
    cpu.next();
    read(addr)
}

pub fn rts_addr_pch(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16;
    cpu.next();
    read(0x0100 | cpu.sp as u16)
}

pub fn rts_done(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.pc = cpu.base_addr | ((cpu.data_latch as u16) << 8);
    let addr = cpu.pc;
    cpu.pc = cpu.pc.wrapping_add(1);
    cpu.next();
    read(addr)
}

// ==========================================================================
// RTI
// ==========================================================================

pub fn rti_dummy(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next();
    read(cpu.pc)
}

pub fn rti_inc_sp(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_add(1);
    cpu.next();
    read(addr)
}

pub fn rti_addr_p(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_add(1);
    cpu.next();
    read(addr)
}

pub fn rti_addr_pcl(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.p = (cpu.data_latch & !(B | U)) | U;
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_add(1);
    cpu.next();
    read(addr)
}

pub fn rti_addr_pch(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16;
    cpu.next();
    read(0x0100 | cpu.sp as u16)
}

pub fn rti_done(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.pc = cpu.base_addr | ((cpu.data_latch as u16) << 8);
    cpu.setup_opcode_fetch()
}

// ==========================================================================
// BRK / IRQ / NMI / RESET
// ==========================================================================

pub fn brk_t0(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = cpu.pc;
    if cpu.brk_flags == 0 {
        cpu.brk_flags = BRK_SOFTWARE;
        cpu.pc = cpu.pc.wrapping_add(1);
    }
    cpu.next();
    read(addr)
}

pub fn brk_t1(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_sub(1);
    cpu.next();
    if cpu.brk_flags & BRK_RESET != 0 {
        read(addr)
    } else {
        write(addr, (cpu.pc >> 8) as u8)
    }
}

pub fn brk_t2(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_sub(1);
    cpu.next();
    if cpu.brk_flags & BRK_RESET != 0 {
        read(addr)
    } else {
        write(addr, cpu.pc as u8)
    }
}

pub fn brk_t3(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    let b = if cpu.brk_flags & BRK_SOFTWARE != 0 { B | U } else { U };
    let data = (cpu.p & !(B | U)) | b;
    cpu.sp = cpu.sp.wrapping_sub(1);
    cpu.p |= I;
    cpu.next();
    if cpu.brk_flags & BRK_RESET != 0 {
        read(addr)
    } else {
        write(addr, data)
    }
}

pub fn brk_t4(cpu: &mut Mos6502) -> Mos6502Output {
    let vector = match cpu.brk_flags {
        f if f & BRK_NMI != 0 => 0xFFFA,
        f if f & BRK_RESET != 0 => 0xFFFC,
        _ => 0xFFFE,
    };
    cpu.next();
    read(vector)
}

pub fn brk_t5(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16;
    let vector = match cpu.brk_flags {
        f if f & BRK_NMI != 0 => 0xFFFB,
        f if f & BRK_RESET != 0 => 0xFFFD,
        _ => 0xFFFF,
    };
    cpu.next();
    read(vector)
}

pub fn brk_t6(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.pc = cpu.base_addr | ((cpu.data_latch as u16) << 8);
    cpu.setup_opcode_fetch()
}

// ==========================================================================
// Stack: PHA, PHP, PLA, PLP
// ==========================================================================

pub fn push_dummy(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next();
    read(cpu.pc)
}

pub fn pha_push(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_sub(1);
    cpu.next();
    write(addr, cpu.a)
}

pub fn php_push(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_sub(1);
    cpu.next();
    write(addr, cpu.p | B | U)
}

pub fn pull_dummy(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next();
    read(cpu.pc)
}

pub fn pull_inc_sp(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_add(1);
    cpu.next();
    read(addr)
}

pub fn addr_pull(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next();
    read(0x0100 | cpu.sp as u16)
}

pub fn pla_done(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.a = cpu.data_latch;
    cpu.set_nz(cpu.a);
    cpu.setup_opcode_fetch()
}

pub fn plp_done(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.p = (cpu.data_latch & !(B | U)) | U;
    cpu.setup_opcode_fetch()
}
