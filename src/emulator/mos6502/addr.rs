/// Micro-op functions.
///
/// Each micro-op consumes data_latch (from the previous cycle's phi2) and
/// returns bus output pins. Named by what they DO with data_latch.
///
/// fetch_opcode is always the last step. It decodes the opcode from
/// data_latch, sets tstate, increments PC, and outputs read(PC).

use super::flags::*;
use super::ops;
use super::{read, sync_read, write, Mos6502, Mos6502Output,
            BRK_IRQ, BRK_NMI, BRK_RESET, BRK_SOFTWARE};

// ==========================================================================
// Generic building blocks
// ==========================================================================

/// Consume opcode from data_latch. Check interrupts. Set tstate. PC++.
/// Output: read(PC). Always the last step of every instruction.
pub fn fetch_opcode(cpu: &mut Mos6502) -> Mos6502Output {
    if cpu.nmi_pending {
        cpu.nmi_pending = false;
        cpu.brk_flags = BRK_NMI;
        cpu.tstate = 0;
    } else if cpu.irq_latch && (cpu.p & I) == 0 {
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
pub fn fetch_data<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    ops::execute_read::<OP>(cpu, cpu.data_latch);
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

/// Save data_latch into base_addr. Output: read(base_addr).
/// Used for ZP operand fetch and (Indirect),Y pointer fetch.
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

/// Combine data_latch as high byte with base_addr low byte. Output: read(base_addr).
/// Used for absolute addr high byte and (Indirect,X) target high byte.
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
pub fn implied<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    ops::execute_implied::<OP>(cpu);
    cpu.next_state();
    sync_read(cpu.pc)
}

/// Consume dummy. Execute accumulator RMW. Output: sync_read(PC).
pub fn accumulator<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    ops::execute_accumulator::<OP>(cpu);
    cpu.next_state();
    sync_read(cpu.pc)
}

// ==========================================================================
// Zero page write
// ==========================================================================

/// Consume ZP address from data_latch. Write register. PC++.
pub fn write_zp<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = cpu.data_latch as u16;
    cpu.inc_pc();
    cpu.next_state();
    write(addr, ops::store_value::<OP>(cpu))
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

/// Consume dummy. Write register to indexed ZP address. PC++.
pub fn write_zp_indexed<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.inc_pc();
    cpu.next_state();
    write(cpu.base_addr, ops::store_value::<OP>(cpu))
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
pub fn write_abs<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr |= (cpu.data_latch as u16) << 8;
    cpu.inc_pc();
    cpu.next_state();
    write(cpu.base_addr, ops::store_value::<OP>(cpu))
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

/// Consume dummy. Write to base_addr (with page cross fixup). PC++.
pub fn fixup_write<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.inc_pc();
    cpu.next_state();
    write(cpu.base_addr, ops::store_value::<OP>(cpu))
}

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

/// Consume target high byte. Form target. Write. PC++.
pub fn write_ind<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr |= (cpu.data_latch as u16) << 8;
    cpu.inc_pc();
    cpu.next_state();
    write(cpu.base_addr, ops::store_value::<OP>(cpu))
}

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

/// Consume value from memory. Dummy write original. Compute modified. PC++.
pub fn rmw_modify<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    let original = cpu.data_latch;
    cpu.rmw_result = ops::execute_rmw::<OP>(cpu, original);
    cpu.inc_pc();
    cpu.next_state();
    write(cpu.base_addr, original)
}

/// Write modified value to base_addr.
pub fn rmw_write(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    write(cpu.base_addr, cpu.rmw_result)
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

/// BRK T0: For software BRK, skip signature byte (PC++). For interrupt, don't.
pub fn brk_t0(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = cpu.pc;
    if cpu.brk_flags == 0 {
        cpu.brk_flags = BRK_SOFTWARE;
        cpu.inc_pc();
    }
    cpu.next_state();
    read(addr)
}

pub fn brk_push_pch(cpu: &mut Mos6502) -> Mos6502Output {
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
    let b = if cpu.brk_flags & BRK_SOFTWARE != 0 { B | U } else { U };
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
pub fn stack_push<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_sub(1);
    cpu.next_state();
    write(addr, ops::push_value::<OP>(cpu))
}

/// Read from stack[SP].
pub fn pull_read(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    read(0x0100 | cpu.sp as u16)
}

/// Consume pulled value. Load register/flags. Output: sync_read(PC).
pub fn stack_pull<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    ops::pull_done::<OP>(cpu, cpu.data_latch);
    cpu.next_state();
    sync_read(cpu.pc)
}
