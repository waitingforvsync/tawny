/// Micro-op functions.
///
/// Each micro-op consumes data_latch (from the previous cycle's phi2) and
/// returns bus output pins. Named by what they DO with data_latch.
///
/// fetch_opcode is always the last step. It decodes the opcode from
/// data_latch, sets tstate, increments PC, and outputs read(PC).

use super::flags::*;
use super::ops::{ReadOp, StoreOp, RmwOp, ImpliedOp, PushOp, PullOp};
use super::{read, sync_read, write, Mos6502, Mos6502Output,
            BRK_IRQ, BRK_NMI, BRK_RESET};

// ==========================================================================
// Generic building blocks
// ==========================================================================

/// Consume opcode from data_latch. Check interrupts. Set tstate. PC++.
/// Output: read(PC). Always the last step of every instruction.
pub fn fetch_opcode(cpu: &mut Mos6502) -> Mos6502Output {
    if cpu.int_shift & 0x20 != 0 {
        cpu.int_shift &= !0x02; // clear NMI pending latch (bit 1)
        cpu.brk_flags = BRK_NMI;
        cpu.tstate = 0;
    } else if cpu.int_shift & 0x10 != 0 {
        cpu.brk_flags = BRK_IRQ;
        cpu.tstate = 0;
    } else {
        cpu.brk_flags = 0;
        cpu.tstate = (cpu.data_latch as u16) << 3;
        cpu.inc_pc();
    }
    read(cpu.pc)
}

/// Consume data value from data_latch. Execute ALU op. PC++.
/// Output: sync_read(PC). Used as the final read step for all addressing modes.
pub fn fetch_data<OP: ReadOp>(cpu: &mut Mos6502) -> Mos6502Output {
    OP::execute(cpu, cpu.data_latch);
    cpu.inc_pc();
    cpu.next_state();
    sync_read(cpu.pc)
}

/// Consume dummy (ignored). Output: sync_read(PC).
/// Used after write cycles and as branch_fixup.
pub fn opcode_read(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    sync_read(cpu.pc)
}

/// Consume dummy (ignored). Output: read(PC). Dummy read from PC.
/// Used by RTS, RTI, PHA, PHP, PLA, PLP, JSR.
pub fn dummy_read(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    read(cpu.pc)
}

/// Output: read(data_latch as ZP address). No state change.
/// Used for ZP read modes where base_addr isn't needed later.
pub fn read_zp(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    read(cpu.data_latch as u16)
}

/// Save data_latch into base_addr. Output: read(base_addr).
/// Used when base_addr is needed by later steps (ZP indexed, (Indirect),Y, ZP RMW).
pub fn latch_to_base(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16;
    cpu.next_state();
    read(cpu.base_addr)
}

/// Output: read(base_addr). Consume dummy (ignored).
/// Used for ZP indexed read, (Indirect,X) pointer read, and fixup after page cross.
pub fn read_base(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    read(cpu.base_addr)
}

/// Combine data_latch as high byte with base_addr low byte. Output: read(full address).
/// Does not store the result — used when base_addr isn't needed later (abs read, ind_x read).
pub fn read_base_hi(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    read(cpu.base_addr | (cpu.data_latch as u16) << 8)
}

/// Combine data_latch as high byte with base_addr low byte. Store in base_addr. Output: read(base_addr).
/// Used when base_addr is needed by later steps (JMP (ind), abs RMW, ind_x write).
pub fn latch_to_base_hi(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr |= (cpu.data_latch as u16) << 8;
    cpu.next_state();
    read(cpu.base_addr)
}

/// Set PC from base_addr | (data_latch << 8). Output: sync_read(PC).
/// Used for JMP abs, JMP (ind), JSR done, RTI, BRK vector.
pub fn latch_to_pc(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.pc = cpu.base_addr | ((cpu.data_latch as u16) << 8);
    cpu.next_state();
    sync_read(cpu.pc)
}

/// Save data_latch as low byte into base_addr. Output: read(stack).
/// Used for RTS PCL and RTI PCL.
pub fn latch_to_base_read_stack(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16;
    cpu.next_state();
    read(0x0100 | cpu.sp as u16)
}

/// Consume dummy. Increment SP. Output: read(stack).
/// Used for RTS and RTI stack traversal.
pub fn inc_sp_read_stack(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_add(1);
    cpu.next_state();
    read(addr)
}

// ==========================================================================
// Implied / Accumulator
// ==========================================================================

/// Consume dummy. Execute implied op. Output: sync_read(PC).
pub fn implied<OP: ImpliedOp>(cpu: &mut Mos6502) -> Mos6502Output {
    OP::execute(cpu);
    cpu.next_state();
    sync_read(cpu.pc)
}

/// Consume dummy. Execute accumulator RMW. Output: sync_read(PC).
pub fn accumulator<OP: RmwOp>(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.a = OP::execute(cpu, cpu.a);
    cpu.next_state();
    sync_read(cpu.pc)
}

// ==========================================================================
// Zero page write
// ==========================================================================

/// Consume ZP address from data_latch. Write register. PC++.
pub fn write_zp<OP: StoreOp>(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = cpu.data_latch as u16;
    cpu.inc_pc();
    cpu.next_state();
    write(addr, OP::value(cpu))
}

// ==========================================================================
// Zero page indexed
// ==========================================================================

/// Consume ZP address. Add X (wrapping in ZP). Output: read(unindexed addr).
/// Used for (Indirect,X) pointer and ZP,X write/RMW modes.
pub fn index_zp_x(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = cpu.data_latch as u16;
    cpu.base_addr = cpu.data_latch.wrapping_add(cpu.x) as u16;
    cpu.next_state();
    read(addr)
}

/// Consume ZP address. Add Y (wrapping in ZP). Output: read(unindexed addr).
/// Used for ZP,Y write mode.
pub fn index_zp_y(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = cpu.data_latch as u16;
    cpu.base_addr = cpu.data_latch.wrapping_add(cpu.y) as u16;
    cpu.next_state();
    read(addr)
}

/// Consume dummy (wasted read from unindexed ZP). Add X to base_addr (wrapping in ZP).
/// Output: read(base_addr). Used for ZP,X read mode.
pub fn add_index_x(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = (cpu.base_addr as u8).wrapping_add(cpu.x) as u16;
    cpu.next_state();
    read(cpu.base_addr)
}

/// Consume dummy (wasted read from unindexed ZP). Add Y to base_addr (wrapping in ZP).
/// Output: read(base_addr). Used for ZP,Y read mode.
pub fn add_index_y(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = (cpu.base_addr as u8).wrapping_add(cpu.y) as u16;
    cpu.next_state();
    read(cpu.base_addr)
}

/// Write register to base_addr. PC++.
/// Used for ZP indexed write, absolute indexed write, and (Indirect),Y write.
pub fn write_base<OP: StoreOp>(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.inc_pc();
    cpu.next_state();
    write(cpu.base_addr, OP::value(cpu))
}

// ==========================================================================
// Absolute
// ==========================================================================

/// Consume addr low byte. Save it. PC++. Output: read(PC) for high byte.
pub fn fetch_addr_lo(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16;
    cpu.inc_pc();
    cpu.next_state();
    read(cpu.pc)
}

/// Consume addr high byte. Form complete address. Write. PC++.
pub fn write_abs<OP: StoreOp>(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr |= (cpu.data_latch as u16) << 8;
    cpu.inc_pc();
    cpu.next_state();
    write(cpu.base_addr, OP::value(cpu))
}

// ==========================================================================
// Absolute indexed
// ==========================================================================

/// Consume addr low byte. Add X. Store as u16 (bit 8 = page cross). PC++.
/// Output: read(PC) for high byte.
pub fn fetch_addr_lo_x(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16 + cpu.x as u16;
    cpu.inc_pc();
    cpu.next_state();
    read(cpu.pc)
}

/// Consume addr low byte. Add Y. Store as u16 (bit 8 = page cross). PC++.
/// Output: read(PC) for high byte.
pub fn fetch_addr_lo_y(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16 + cpu.y as u16;
    cpu.inc_pc();
    cpu.next_state();
    read(cpu.pc)
}

/// Consume addr high byte. Form indexed address. If no page cross, skip fixup.
/// Output: read(indexed addr, maybe wrong page).
pub fn fetch_addr_hi_indexed(cpu: &mut Mos6502) -> Mos6502Output {
    let page_crossed = (cpu.base_addr & 0x100) != 0;
    cpu.base_addr = cpu.base_addr.wrapping_add((cpu.data_latch as u16) << 8);

    if page_crossed {
        cpu.next_state();
        read(cpu.base_addr.wrapping_sub(0x100))
    } else {
        cpu.skip_next_state();
        read(cpu.base_addr)
    }
}

/// Like fetch_addr_hi_indexed but always takes penalty. Preserves bit 8 for fixup.
pub fn fetch_addr_hi_indexed_penalty(cpu: &mut Mos6502) -> Mos6502Output {
    let msb = (cpu.data_latch as u16) << 8;
    let addr = (cpu.base_addr & 0xFF) | msb;
    cpu.base_addr = cpu.base_addr.wrapping_add(msb);
    cpu.next_state();
    read(addr)
}

// fixup_write = write_base (same: write to base_addr, PC++)

// ==========================================================================
// (Indirect,X)
// ==========================================================================

/// Consume target low byte. Save it. Output: read(ZP ptr + 1, wrapping in ZP).
pub fn fetch_ind_lo(cpu: &mut Mos6502) -> Mos6502Output {
    let ptr_lo = cpu.base_addr;
    cpu.base_addr = cpu.data_latch as u16;
    cpu.next_state();
    read((ptr_lo.wrapping_add(1)) & 0x00FF)
}

// write_ind = write_abs (same: merge high byte, write to base_addr, PC++)

// ==========================================================================
// (Indirect),Y
// ==========================================================================

/// Consume base low byte. Add Y (bit 8 = page cross).
/// Output: read(ZP ptr + 1, wrapping in ZP).
pub fn fetch_ind_y_lo(cpu: &mut Mos6502) -> Mos6502Output {
    let ptr_lo = cpu.base_addr;
    cpu.base_addr = cpu.data_latch as u16 + cpu.y as u16;
    cpu.next_state();
    read((ptr_lo.wrapping_add(1)) & 0x00FF)
}

// After fetch_ind_y_lo, reuses fetch_addr_hi_indexed / _penalty + fixup.

// ==========================================================================
// Read-modify-write
// ==========================================================================

/// Dummy-write original value back to base_addr. PC++.
pub fn rmw_dummy_write(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.inc_pc();
    cpu.next_state();
    write(cpu.base_addr, cpu.data_latch)
}

/// Execute RMW op on data_latch (the original value, re-latched from dummy write).
/// Write modified result to base_addr.
pub fn rmw_execute<OP: RmwOp>(cpu: &mut Mos6502) -> Mos6502Output {
    let result = OP::execute(cpu, cpu.data_latch);
    cpu.next_state();
    write(cpu.base_addr, result)
}

/// JAM: CPU halts. Does not advance tstate — stays frozen.
pub fn jam(cpu: &mut Mos6502) -> Mos6502Output {
    read(cpu.pc)
}

/// TAS helper: set SP = A & X, then write A & X & (addr_hi + 1).
/// Used as the fixup_write step for TAS ($9B).
pub fn tas_write(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.sp = cpu.a & cpu.x;
    let val = cpu.a & cpu.x & ((cpu.base_addr >> 8) as u8).wrapping_add(1);
    cpu.inc_pc();
    cpu.next_state();
    write(cpu.base_addr, val)
}

// ==========================================================================
// Branches
// ==========================================================================

/// Consume branch offset. Check condition. PC++.
pub fn branch<const FLAG: u8, const SET: bool>(cpu: &mut Mos6502) -> Mos6502Output {
    let taken = if SET { cpu.p & FLAG != 0 } else { cpu.p & FLAG == 0 };
    cpu.inc_pc();
    if taken {
        cpu.base_addr = cpu.data_latch as u16;
        cpu.next_state();
        read(cpu.pc)
    } else {
        cpu.tstate += 3;
        sync_read(cpu.pc)
    }
}

/// Consume dummy. Apply branch offset. Check page cross.
pub fn branch_take(cpu: &mut Mos6502) -> Mos6502Output {
    let offset = cpu.base_addr as u8 as i8;
    let old_pc = cpu.pc;
    cpu.pc = cpu.pc.wrapping_add(offset as i16 as u16);
    if (old_pc ^ cpu.pc) & 0xFF00 != 0 {
        cpu.next_state();
        read((old_pc & 0xFF00) | (cpu.pc & 0x00FF))
    } else {
        cpu.skip_next_state();
        sync_read(cpu.pc)
    }
}

// branch_fixup = opcode_read (already defined above)

// ==========================================================================
// JMP
// ==========================================================================

/// JMP (ind): Consume target low byte. NMOS page-wrap bug.
/// Output: read(pointer+1, wrapping within page).
pub fn jmp_ind_lo(cpu: &mut Mos6502) -> Mos6502Output {
    let target_lo = cpu.data_latch;
    let hi_addr = (cpu.base_addr & 0xFF00) | ((cpu.base_addr.wrapping_add(1)) & 0x00FF);
    cpu.base_addr = target_lo as u16;
    cpu.next_state();
    read(hi_addr)
}

// ==========================================================================
// JSR
// ==========================================================================

/// Consume addr low byte. Save it. PC++. Output: read(stack) — dummy.
pub fn jsr_save_lo(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16;
    cpu.inc_pc();
    cpu.next_state();
    read(0x0100 | cpu.sp as u16)
}

/// Push PCH to stack.
pub fn jsr_push_pch(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_sub(1);
    cpu.next_state();
    write(addr, (cpu.pc >> 8) as u8)
}

/// Push PCL to stack.
pub fn jsr_push_pcl(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_sub(1);
    cpu.next_state();
    write(addr, cpu.pc as u8)
}

// jsr_fetch_hi = dummy_read (reads from PC, which points at the high byte)

// ==========================================================================
// RTS
// ==========================================================================

/// RTS: Consume PCH. Form PC. PC++ (JSR pushed last byte addr).
/// Output: sync_read(PC).
pub fn rts_read_pch(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.pc = cpu.base_addr | ((cpu.data_latch as u16) << 8);
    cpu.inc_pc();
    cpu.next_state();
    sync_read(cpu.pc)
}

// ==========================================================================
// RTI
// ==========================================================================

/// Consume P from data_latch. Restore P. Increment SP. Output: read(stack).
pub fn rti_read_p(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.p = (cpu.data_latch & !(B | U)) | U;
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_add(1);
    cpu.next_state();
    read(addr)
}

// ==========================================================================
// BRK / IRQ / NMI / RESET
// ==========================================================================

pub fn brk_push_pch(cpu: &mut Mos6502) -> Mos6502Output {
    // Software BRK (brk_flags == 0): skip signature byte.
    if cpu.brk_flags == 0 {
        cpu.inc_pc();
    }
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_sub(1);
    cpu.next_state();
    if cpu.brk_flags & BRK_RESET != 0 {
        read(addr)
    } else {
        write(addr, (cpu.pc >> 8) as u8)
    }
}

pub fn brk_push_pcl(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_sub(1);
    cpu.next_state();
    if cpu.brk_flags & BRK_RESET != 0 {
        read(addr)
    } else {
        write(addr, cpu.pc as u8)
    }
}

pub fn brk_push_p(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    let b = if cpu.brk_flags == 0 { B | U } else { U };
    let data = (cpu.p & !(B | U)) | b;
    cpu.sp = cpu.sp.wrapping_sub(1);
    cpu.p |= I;
    cpu.next_state();
    if cpu.brk_flags & BRK_RESET != 0 {
        read(addr)
    } else {
        write(addr, data)
    }
}

pub fn brk_vector_lo(cpu: &mut Mos6502) -> Mos6502Output {
    let vector = match cpu.brk_flags {
        f if f & BRK_NMI != 0 => 0xFFFA,
        f if f & BRK_RESET != 0 => 0xFFFC,
        _ => 0xFFFE,
    };
    cpu.next_state();
    read(vector)
}

/// Consume vector low byte. Fetch vector high byte.
pub fn brk_read_vector_lo(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16;
    let vector = match cpu.brk_flags {
        f if f & BRK_NMI != 0 => 0xFFFB,
        f if f & BRK_RESET != 0 => 0xFFFD,
        _ => 0xFFFF,
    };
    cpu.next_state();
    read(vector)
}

// brk_read_vector_hi = latch_to_pc

// ==========================================================================
// Stack: PHA, PHP, PLA, PLP
// ==========================================================================

/// Push register/flags to stack. Decrement SP.
pub fn stack_push<OP: PushOp>(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_sub(1);
    cpu.next_state();
    write(addr, OP::value(cpu))
}

/// Read from stack[SP].
pub fn pull_read(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    read(0x0100 | cpu.sp as u16)
}

/// Consume pulled value. Load register/flags. Output: sync_read(PC).
pub fn stack_pull<OP: PullOp>(cpu: &mut Mos6502) -> Mos6502Output {
    OP::execute(cpu, cpu.data_latch);
    cpu.next_state();
    sync_read(cpu.pc)
}
