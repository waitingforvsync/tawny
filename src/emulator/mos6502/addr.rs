/// Micro-op functions.
///
/// Each micro-op consumes data_latch (from the previous cycle's phi2) and
/// returns bus output pins. Named by what they DO with data_latch.
///
/// fetch_opcode is always the last step. It decodes the opcode from
/// data_latch, sets tstate, increments PC, and outputs read(PC).
///
/// PC is incremented by whichever micro-op consumed a byte that was
/// fetched from PC. Steps that consume data from computed addresses
/// (ZP, absolute, stack) do not increment PC.

use super::flags::*;
use super::ops;
use super::{read, sync_read, write, Mos6502, Mos6502Output,
            BRK_IRQ, BRK_NMI, BRK_RESET, BRK_SOFTWARE};

// ==========================================================================
// fetch_opcode — always the last step of every instruction
// ==========================================================================

/// Consume opcode from data_latch. Check interrupts. Set tstate. PC++.
/// Output: read(PC) — the first operand byte of the new instruction.
pub fn fetch_opcode(cpu: &mut Mos6502) -> Mos6502Output {
    if cpu.nmi_pending {
        cpu.nmi_pending = false;
        cpu.brk_flags = BRK_NMI;
        // Don't advance PC — we want to return to this address after RTI.
        cpu.tstate = 0;
    } else if cpu.irq_latch && (cpu.p & I) == 0 {
        cpu.brk_flags = BRK_IRQ;
        cpu.tstate = 0;
    } else {
        cpu.brk_flags = 0;
        cpu.tstate = (cpu.data_latch as u16) << 3;
    }
    cpu.inc_pc();
    read(cpu.pc)
}

// ==========================================================================
// Immediate: [fetch_imm<OP>, fetch_opcode]
// ==========================================================================

/// Consume immediate value from data_latch. Execute ALU op. PC++.
/// Output: sync_read(PC) — the next opcode.
pub fn fetch_imm<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    ops::execute_read::<OP>(cpu);
    cpu.inc_pc();
    cpu.next_state();
    sync_read(cpu.pc)
}

// ==========================================================================
// Implied / Accumulator: [implied<OP>, fetch_opcode]
// ==========================================================================

/// Consume dummy from data_latch (ignored). Execute implied op.
/// Output: sync_read(PC) — the next opcode. No PC++ (already done by fetch_opcode).
pub fn implied<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    ops::execute_implied::<OP>(cpu);
    cpu.next_state();
    sync_read(cpu.pc)
}

/// Consume dummy. Execute accumulator RMW.
/// Output: sync_read(PC). No PC++.
pub fn accumulator<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    ops::execute_accumulator::<OP>(cpu);
    cpu.next_state();
    sync_read(cpu.pc)
}

// ==========================================================================
// Zero page read: [fetch_zp_operand, fetch_zp<OP>, fetch_opcode]
// ==========================================================================

/// Consume ZP address from data_latch. PC++.
/// Output: read(zp_addr) — the value at the ZP address.
pub fn fetch_zp_operand(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16;
    cpu.inc_pc();
    cpu.next_state();
    read(cpu.base_addr)
}

/// Consume value from ZP. Execute ALU op. No PC++.
/// Output: sync_read(PC) — the next opcode.
pub fn fetch_zp<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    ops::execute_read::<OP>(cpu);
    cpu.next_state();
    sync_read(cpu.pc)
}

// ==========================================================================
// Zero page write: [fetch_zp_operand, write_zp<OP>, opcode_read, fetch_opcode]
// ==========================================================================

/// Consume ZP address from data_latch. Write register to ZP. PC++.
/// Output: write(zp_addr, value).
pub fn write_zp<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = cpu.data_latch as u16;
    cpu.inc_pc();
    cpu.next_state();
    write(addr, ops::store_value::<OP>(cpu))
}

/// After a write cycle, read the next opcode. No PC++.
/// Output: sync_read(PC) — the next opcode.
pub fn opcode_read(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    sync_read(cpu.pc)
}

// ==========================================================================
// Zero page indexed read: [index_zp_x, addr_zp_indexed, fetch_zp<OP>, fetch_opcode]
// ==========================================================================

/// Consume ZP address from data_latch. Add X (wrapping in ZP). PC++.
/// Output: read(unindexed ZP addr) — dummy read.
pub fn index_zp_x(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = cpu.data_latch as u16;
    cpu.base_addr = cpu.data_latch.wrapping_add(cpu.x) as u16;
    cpu.inc_pc();
    cpu.next_state();
    read(addr)
}

/// Consume ZP address from data_latch. Add Y (wrapping in ZP). PC++.
/// Output: read(unindexed ZP addr) — dummy read.
pub fn index_zp_y(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = cpu.data_latch as u16;
    cpu.base_addr = cpu.data_latch.wrapping_add(cpu.y) as u16;
    cpu.inc_pc();
    cpu.next_state();
    read(addr)
}

/// Consume dummy. Output: read(base_addr) — the indexed ZP address.
pub fn addr_zp_indexed(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    read(cpu.base_addr)
}

// Zero page indexed write: [index_zp_x, write_zp_indexed<OP>, opcode_read, fetch_opcode]

/// Consume dummy. Write register to indexed ZP address.
/// Output: write(base_addr, value).
pub fn write_zp_indexed<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    write(cpu.base_addr, ops::store_value::<OP>(cpu))
}

// ==========================================================================
// Absolute read: [fetch_addr_lo, fetch_addr_hi, fetch_abs<OP>, fetch_opcode]
// ==========================================================================

/// Consume addr low byte from data_latch. PC++.
/// Output: read(PC) — to fetch the high byte. PC++.
pub fn fetch_addr_lo(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16;
    cpu.inc_pc();
    cpu.next_state();
    read(cpu.pc)
}

/// Consume addr high byte from data_latch. Form complete address. PC++.
/// Output: read(abs_addr) — the value at the absolute address.
pub fn fetch_addr_hi(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr |= (cpu.data_latch as u16) << 8;
    cpu.inc_pc();
    cpu.next_state();
    read(cpu.base_addr)
}

/// Consume value from absolute address. Execute ALU op. No PC++.
/// Output: sync_read(PC) — the next opcode.
pub fn fetch_abs<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    ops::execute_read::<OP>(cpu);
    cpu.next_state();
    sync_read(cpu.pc)
}

// Absolute write: [fetch_addr_lo, write_abs<OP>, opcode_read, fetch_opcode]

/// Consume addr high byte. Form complete address. Write. PC++.
/// Output: write(abs_addr, value).
pub fn write_abs<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr |= (cpu.data_latch as u16) << 8;
    cpu.inc_pc();
    cpu.next_state();
    write(cpu.base_addr, ops::store_value::<OP>(cpu))
}

// ==========================================================================
// Absolute indexed read: [fetch_addr_lo_x, fetch_addr_hi_indexed,
//                          fixup_indexed, fetch_abs<OP>, fetch_opcode]
// ==========================================================================

/// Consume addr low byte. Add X. Store as u16 (bit 8 = page cross). PC++.
/// Output: read(PC) — to fetch high byte. PC++.
pub fn fetch_addr_lo_x(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16 + cpu.x as u16;
    cpu.inc_pc();
    cpu.next_state();
    read(cpu.pc)
}

/// Consume addr low byte. Add Y. Store as u16 (bit 8 = page cross). PC++.
/// Output: read(PC) — to fetch high byte. PC++.
pub fn fetch_addr_lo_y(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16 + cpu.y as u16;
    cpu.inc_pc();
    cpu.next_state();
    read(cpu.pc)
}

/// Consume addr high byte. Form indexed address (low byte from base_addr,
/// bit 8 = page cross). PC++.
/// If no page cross, skip fixup step.
/// Output: read(indexed addr, maybe wrong page — high byte not yet corrected).
pub fn fetch_addr_hi_indexed(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.inc_pc();
    let page_crossed = (cpu.base_addr & 0x100) != 0;
    cpu.base_addr = cpu.base_addr.wrapping_add((cpu.data_latch as u16) << 8);

    if page_crossed {
        // Page cross: proceed to fixup.
        cpu.next_state();
        read(cpu.base_addr.wrapping_sub(0x100))
    } else {
        // No page cross: skip fixup. Address is correct.
        cpu.skip_next_state();
        read(cpu.base_addr)
    }
}

/// Consume addr high byte. Form indexed address (without fixing page cross). PC++.
/// Always take penalty — fixup step follows.
/// Preserves bit 8 of base_addr for fixup to detect page cross.
/// Output: read(indexed addr, maybe wrong page).
pub fn fetch_addr_hi_indexed_penalty(cpu: &mut Mos6502) -> Mos6502Output {
    let msb = (cpu.data_latch as u16) << 8;
    let addr = (cpu.base_addr & 0xFF) | msb;
    // Keep base_addr with carry info: high byte from data_latch + original bit 8.
    cpu.base_addr = cpu.base_addr.wrapping_add(msb);
    cpu.inc_pc();
    cpu.next_state();
    read(addr)
}

/// Consume dummy. Fix high byte if page crossed (bit 8 of base_addr).
/// Output: read(corrected addr).
pub fn fixup_indexed(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    read(cpu.base_addr)
}

// Absolute indexed write: [fetch_addr_lo_x, fetch_addr_hi_indexed_penalty,
//                           fixup_write<OP>, opcode_read, fetch_opcode]

/// Consume dummy. Fix high byte if page crossed (bit 8 of base_addr). Write.
/// Output: write(corrected addr, value).
pub fn fixup_write<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    write(cpu.base_addr, ops::store_value::<OP>(cpu))
}

// ==========================================================================
// (Indirect,X) read: [fetch_ind_x_ptr, addr_ind_x_ptr, fetch_ind_lo,
//                      fetch_ind_hi, fetch_abs<OP>, fetch_opcode]
// ==========================================================================

/// Consume ZP pointer from data_latch. Add X. PC++.
/// Output: read(unindexed ZP addr) — dummy.
pub fn fetch_ind_x_ptr(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = cpu.data_latch as u16;
    cpu.base_addr = cpu.data_latch.wrapping_add(cpu.x) as u16;
    cpu.inc_pc();
    cpu.next_state();
    read(addr)
}

/// Consume dummy. Read target low byte from ZP pointer.
/// Output: read(base_addr).
pub fn addr_ind_x_ptr(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    read(cpu.base_addr)
}

/// Consume target low byte from data_latch. Save it.
/// Output: read(ZP pointer + 1, wrapping in ZP) — target high byte.
pub fn fetch_ind_lo(cpu: &mut Mos6502) -> Mos6502Output {
    let ptr_lo = cpu.base_addr;
    cpu.base_addr = cpu.data_latch as u16;
    cpu.next_state();
    read((ptr_lo.wrapping_add(1)) & 0x00FF)
}

/// Consume target high byte from data_latch. Form complete target address.
/// Output: read(target) — the value at the target.
pub fn fetch_ind_hi(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr |= (cpu.data_latch as u16) << 8;
    cpu.next_state();
    read(cpu.base_addr)
}

// (Indirect,X) write: [..., write_ind<OP>, opcode_read, fetch_opcode]

/// Consume target high byte. Form target. Write.
/// Output: write(target, value).
pub fn write_ind<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr |= (cpu.data_latch as u16) << 8;
    cpu.next_state();
    write(cpu.base_addr, ops::store_value::<OP>(cpu))
}

// ==========================================================================
// (Indirect),Y read: [fetch_ind_y_ptr, fetch_ind_y_lo,
//                      fetch_addr_hi_indexed, fixup_indexed,
//                      fetch_abs<OP>, fetch_opcode]
// ==========================================================================

/// Consume ZP pointer from data_latch. PC++.
/// Output: read(ZP ptr) — low byte of base address.
pub fn fetch_ind_y_ptr(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16;
    cpu.inc_pc();
    cpu.next_state();
    read(cpu.base_addr)
}

/// Consume base low byte. Add Y. Store as u16 (bit 8 = page cross).
/// Output: read(ZP ptr + 1, wrapping in ZP) — high byte of base address.
pub fn fetch_ind_y_lo(cpu: &mut Mos6502) -> Mos6502Output {
    let ptr_lo = cpu.base_addr;
    cpu.base_addr = cpu.data_latch as u16 + cpu.y as u16;
    cpu.next_state();
    read((ptr_lo.wrapping_add(1)) & 0x00FF)
}

/// Consume addr high byte for (Indirect),Y. Form indexed address.
/// No PC++ — PC is already past the operand.
/// If no page cross, skip fixup step.
pub fn fetch_ind_y_hi(cpu: &mut Mos6502) -> Mos6502Output {
    let page_crossed = (cpu.base_addr & 0x100) != 0;
    cpu.base_addr = cpu.base_addr.wrapping_add((cpu.data_latch as u16) << 8);

    if page_crossed {
        // Page cross: proceed to fixup.
        cpu.next_state();
        read(cpu.base_addr.wrapping_sub(0x100))
    } else {
        // No page cross: skip fixup. Address is correct.
        cpu.skip_next_state();
        read(cpu.base_addr)
    }
}

/// Like fetch_ind_y_hi but always take penalty (for writes).
/// Preserves bit 8 of base_addr for fixup.
pub fn fetch_ind_y_hi_penalty(cpu: &mut Mos6502) -> Mos6502Output {
    let msb = (cpu.data_latch as u16) << 8;
    let addr = (cpu.base_addr & 0xFF) | msb;
    // Keep base_addr with carry info: high byte from data_latch + original bit 8.
    cpu.base_addr = cpu.base_addr.wrapping_add(msb);
    cpu.next_state();
    read(addr)
}

// ==========================================================================
// Read-modify-write
// ==========================================================================

/// Consume value from memory. Dummy write original value. Compute modified.
/// Output: write(base_addr, original).
pub fn rmw_modify<const OP: u8>(cpu: &mut Mos6502) -> Mos6502Output {
    let original = cpu.data_latch;
    cpu.rmw_result = ops::execute_rmw::<OP>(cpu);
    cpu.next_state();
    write(cpu.base_addr, original)
}

/// Consume nothing (write cycle). Write modified value.
/// Output: write(base_addr, modified).
pub fn rmw_write(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    write(cpu.base_addr, cpu.rmw_result)
}

// ==========================================================================
// Branches
// ==========================================================================

/// Consume branch offset from data_latch. Check condition. PC++.
/// If taken: save offset, dummy read.
/// If not taken: sync_read(PC) for next opcode.
pub fn branch<const FLAG: u8, const SET: bool>(cpu: &mut Mos6502) -> Mos6502Output {
    let taken = if SET { cpu.p & FLAG != 0 } else { cpu.p & FLAG == 0 };
    cpu.inc_pc();
    if taken {
        cpu.base_addr = cpu.data_latch as u16; // save offset
        cpu.next_state(); // proceed to branch_take
        read(cpu.pc) // dummy read
    } else {
        cpu.tstate += 3; // skip branch_take + branch_fixup, land on fetch_opcode
        sync_read(cpu.pc)
    }
}

/// Consume dummy. Apply branch offset. Check page cross.
pub fn branch_take(cpu: &mut Mos6502) -> Mos6502Output {
    let offset = cpu.base_addr as u8 as i8;
    let old_pc = cpu.pc;
    cpu.pc = cpu.pc.wrapping_add(offset as i16 as u16);
    if (old_pc ^ cpu.pc) & 0xFF00 != 0 {
        cpu.next_state(); // proceed to branch_fixup
        read((old_pc & 0xFF00) | (cpu.pc & 0x00FF))
    } else {
        cpu.skip_next_state(); // skip branch_fixup, land on fetch_opcode
        sync_read(cpu.pc)
    }
}

/// Consume dummy. PC is correct. Output: sync_read(PC).
pub fn branch_fixup(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    sync_read(cpu.pc)
}

// ==========================================================================
// JMP
// ==========================================================================

/// JMP abs: Consume addr high byte. Form target. Set PC. No PC++.
/// Output: sync_read(PC) — the next opcode at the target.
pub fn jmp_abs(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.pc = cpu.base_addr | ((cpu.data_latch as u16) << 8);
    cpu.next_state();
    sync_read(cpu.pc)
}

/// JMP (ind): Consume addr high byte. Form pointer.
/// Output: read(pointer) — target low byte.
pub fn jmp_ind_addr(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr |= (cpu.data_latch as u16) << 8;
    cpu.next_state();
    read(cpu.base_addr)
}

/// JMP (ind): Consume target low byte. NMOS page-wrap bug.
/// Output: read(pointer+1, wrapping within page) — target high byte.
pub fn jmp_ind_lo(cpu: &mut Mos6502) -> Mos6502Output {
    let target_lo = cpu.data_latch;
    let hi_addr = (cpu.base_addr & 0xFF00) | ((cpu.base_addr.wrapping_add(1)) & 0x00FF);
    cpu.base_addr = target_lo as u16;
    cpu.next_state();
    read(hi_addr)
}

/// JMP (ind): Consume target high byte. Set PC.
/// Output: sync_read(PC) — opcode at target.
pub fn jmp_ind_hi(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.pc = cpu.base_addr | ((cpu.data_latch as u16) << 8);
    cpu.next_state();
    sync_read(cpu.pc)
}

// ==========================================================================
// JSR
// ==========================================================================

/// JSR: Consume addr low byte. Save it. PC++ (now points at high byte).
/// Output: read(stack) — dummy.
pub fn jsr_save_lo(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16;
    cpu.inc_pc();
    cpu.next_state();
    read(0x0100 | cpu.sp as u16)
}

/// JSR: Consume dummy. Push PCH. (PC points at high byte = return addr.)
/// Output: write(stack, PCH).
pub fn jsr_push_pch(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_sub(1);
    cpu.next_state();
    write(addr, (cpu.pc >> 8) as u8)
}

/// JSR: Push PCL.
/// Output: write(stack, PCL).
pub fn jsr_push_pcl(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_sub(1);
    cpu.next_state();
    write(addr, cpu.pc as u8)
}

/// JSR: Fetch high byte of target from PC (which still points at it).
/// Output: read(PC).
pub fn jsr_fetch_hi(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    read(cpu.pc)
}

/// JSR: Consume addr high byte. Set PC to target.
/// Output: sync_read(PC) — opcode at target.
pub fn jsr_done(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.pc = cpu.base_addr | ((cpu.data_latch as u16) << 8);
    cpu.next_state();
    sync_read(cpu.pc)
}

// ==========================================================================
// RTS
// ==========================================================================

/// RTS: Consume dummy. Dummy read from PC.
pub fn rts_dummy(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    read(cpu.pc)
}

/// RTS: Consume dummy. Dummy stack read, SP++.
pub fn rts_inc_sp(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_add(1);
    cpu.next_state();
    read(addr)
}

/// RTS: Consume dummy. Read PCL from stack, SP++.
pub fn rts_addr_pcl(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_add(1);
    cpu.next_state();
    read(addr)
}

/// RTS: Consume PCL from data_latch. Read PCH from stack.
pub fn rts_read_pcl(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16;
    cpu.next_state();
    read(0x0100 | cpu.sp as u16)
}

/// RTS: Consume PCH from data_latch. Form PC. PC++ (JSR pushed last byte addr).
/// Output: sync_read(PC) — opcode at return address.
pub fn rts_read_pch(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.pc = cpu.base_addr | ((cpu.data_latch as u16) << 8);
    cpu.inc_pc();
    cpu.next_state();
    sync_read(cpu.pc)
}

// ==========================================================================
// RTI
// ==========================================================================

pub fn rti_dummy(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    read(cpu.pc)
}

pub fn rti_inc_sp(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_add(1);
    cpu.next_state();
    read(addr)
}

pub fn rti_addr_p(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_add(1);
    cpu.next_state();
    read(addr)
}

/// Consume P from data_latch. Restore P. Read PCL, SP++.
pub fn rti_read_p(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.p = (cpu.data_latch & !(B | U)) | U;
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_add(1);
    cpu.next_state();
    read(addr)
}

/// Consume PCL. Read PCH.
pub fn rti_read_pcl(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.base_addr = cpu.data_latch as u16;
    cpu.next_state();
    read(0x0100 | cpu.sp as u16)
}

/// Consume PCH. Set PC. Output: sync_read(PC).
pub fn rti_read_pch(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.pc = cpu.base_addr | ((cpu.data_latch as u16) << 8);
    cpu.next_state();
    sync_read(cpu.pc)
}

// ==========================================================================
// BRK / IRQ / NMI / RESET (all use opcode $00)
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

/// Consume vector high byte. Set PC. Output: sync_read(PC).
pub fn brk_read_vector_hi(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.pc = cpu.base_addr | ((cpu.data_latch as u16) << 8);
    cpu.next_state();
    sync_read(cpu.pc)
}

// ==========================================================================
// Stack: PHA, PHP, PLA, PLP
// ==========================================================================

/// Dummy read from PC. No PC++.
pub fn push_dummy(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    read(cpu.pc)
}

pub fn pha_push(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_sub(1);
    cpu.next_state();
    write(addr, cpu.a)
}

pub fn php_push(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_sub(1);
    cpu.next_state();
    write(addr, cpu.p | B | U)
}

pub fn pull_dummy(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    read(cpu.pc)
}

pub fn pull_inc_sp(cpu: &mut Mos6502) -> Mos6502Output {
    let addr = 0x0100 | cpu.sp as u16;
    cpu.sp = cpu.sp.wrapping_add(1);
    cpu.next_state();
    read(addr)
}

pub fn pull_read(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.next_state();
    read(0x0100 | cpu.sp as u16)
}

/// Consume pulled value. Load A, set flags. Output: sync_read(PC).
pub fn pla_done(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.a = cpu.data_latch;
    cpu.set_nz(cpu.a);
    cpu.next_state();
    sync_read(cpu.pc)
}

/// Consume pulled value. Restore P. Output: sync_read(PC).
pub fn plp_done(cpu: &mut Mos6502) -> Mos6502Output {
    cpu.p = (cpu.data_latch & !(B | U)) | U;
    cpu.next_state();
    sync_read(cpu.pc)
}
