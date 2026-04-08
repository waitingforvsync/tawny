/// Micro-op step table for all 6502 instructions.
///
/// Every instruction ends with `fetch_opcode`, which consumes the opcode
/// byte from data_latch, decodes it, and sets tstate for the next instruction.
use super::addr::*;
use super::ops;
use super::{MicroOp, TABLE_SIZE, MAX_STEPS};

pub static STEPS: [MicroOp; TABLE_SIZE] = build_steps();

fn trap(cpu: &mut super::Mos6502) -> super::Mos6502Output {
    // Don't call cpu.next_state() — stay at the same tstate forever.
    super::read(cpu.pc)
}

#[inline(always)]
pub fn dispatch(cpu: &mut super::Mos6502) -> super::Mos6502Output {
    STEPS[cpu.tstate as usize](cpu)
}

const fn set(table: &mut [MicroOp; TABLE_SIZE], opcode: usize, steps: &[MicroOp]) {
    let base = opcode * MAX_STEPS;
    let mut i = 0;
    while i < steps.len() {
        table[base + i] = steps[i];
        i += 1;
    }
}

// ======================================================================
// Addressing mode generators
// ======================================================================

const fn imm_read<OP: ops::ReadOp>() -> [MicroOp; 2] {
    [fetch_data::<OP>, fetch_opcode]
}

const fn zp_read<OP: ops::ReadOp>() -> [MicroOp; 3] {
    [latch_to_base, fetch_data::<OP>, fetch_opcode]
}

const fn zp_write<OP: ops::StoreOp>() -> [MicroOp; 3] {
    [write_zp::<OP>, opcode_read, fetch_opcode]
}

const fn zp_x_read<OP: ops::ReadOp>() -> [MicroOp; 4] {
    [latch_to_base, add_index_x, fetch_data::<OP>, fetch_opcode]
}

const fn zp_y_read<OP: ops::ReadOp>() -> [MicroOp; 4] {
    [latch_to_base, add_index_y, fetch_data::<OP>, fetch_opcode]
}

const fn zp_x_write<OP: ops::StoreOp>() -> [MicroOp; 5] {
    [index_zp_x, read_base, write_zp_indexed::<OP>, opcode_read, fetch_opcode]
}

const fn zp_y_write<OP: ops::StoreOp>() -> [MicroOp; 5] {
    [index_zp_y, read_base, write_zp_indexed::<OP>, opcode_read, fetch_opcode]
}

const fn abs_read<OP: ops::ReadOp>() -> [MicroOp; 4] {
    [fetch_addr_lo, latch_to_base_hi, fetch_data::<OP>, fetch_opcode]
}

const fn abs_write<OP: ops::StoreOp>() -> [MicroOp; 4] {
    [fetch_addr_lo, write_abs::<OP>, opcode_read, fetch_opcode]
}

const fn abs_x_read<OP: ops::ReadOp>() -> [MicroOp; 5] {
    [fetch_addr_lo_x, fetch_addr_hi_indexed, read_base, fetch_data::<OP>, fetch_opcode]
}

const fn abs_y_read<OP: ops::ReadOp>() -> [MicroOp; 5] {
    [fetch_addr_lo_y, fetch_addr_hi_indexed, read_base, fetch_data::<OP>, fetch_opcode]
}

const fn abs_x_write<OP: ops::StoreOp>() -> [MicroOp; 5] {
    [fetch_addr_lo_x, fetch_addr_hi_indexed_penalty, fixup_write::<OP>, opcode_read, fetch_opcode]
}

const fn abs_y_write<OP: ops::StoreOp>() -> [MicroOp; 5] {
    [fetch_addr_lo_y, fetch_addr_hi_indexed_penalty, fixup_write::<OP>, opcode_read, fetch_opcode]
}

const fn ind_x_read<OP: ops::ReadOp>() -> [MicroOp; 6] {
    [index_zp_x, read_base, fetch_ind_lo, latch_to_base_hi, fetch_data::<OP>, fetch_opcode]
}

const fn ind_x_write<OP: ops::StoreOp>() -> [MicroOp; 6] {
    [index_zp_x, read_base, fetch_ind_lo, write_ind::<OP>, opcode_read, fetch_opcode]
}

const fn ind_y_read<OP: ops::ReadOp>() -> [MicroOp; 6] {
    [latch_to_base, fetch_ind_y_lo, fetch_addr_hi_indexed, read_base, fetch_data::<OP>, fetch_opcode]
}

const fn ind_y_write<OP: ops::StoreOp>() -> [MicroOp; 6] {
    [latch_to_base, fetch_ind_y_lo, fetch_addr_hi_indexed_penalty, fixup_write::<OP>, opcode_read, fetch_opcode]
}

const fn acc_rmw<OP: ops::RmwOp>() -> [MicroOp; 2] {
    [accumulator::<OP>, fetch_opcode]
}

const fn zp_rmw<OP: ops::RmwOp>() -> [MicroOp; 5] {
    [latch_to_base, rmw_modify::<OP>, rmw_write, opcode_read, fetch_opcode]
}

const fn zp_x_rmw<OP: ops::RmwOp>() -> [MicroOp; 6] {
    [index_zp_x, read_base, rmw_modify::<OP>, rmw_write, opcode_read, fetch_opcode]
}

const fn abs_rmw<OP: ops::RmwOp>() -> [MicroOp; 6] {
    [fetch_addr_lo, latch_to_base_hi, rmw_modify::<OP>, rmw_write, opcode_read, fetch_opcode]
}

const fn abs_x_rmw<OP: ops::RmwOp>() -> [MicroOp; 7] {
    [fetch_addr_lo_x, fetch_addr_hi_indexed_penalty, read_base, rmw_modify::<OP>, rmw_write, opcode_read, fetch_opcode]
}

const fn imp<OP: ops::ImpliedOp>() -> [MicroOp; 2] {
    [implied::<OP>, fetch_opcode]
}

const fn branch_op<const FLAG: u8, const SET: bool>() -> [MicroOp; 4] {
    [branch::<FLAG, SET>, branch_take, opcode_read, fetch_opcode]
}

const fn push<OP: ops::PushOp>() -> [MicroOp; 4] {
    [dummy_read, stack_push::<OP>, opcode_read, fetch_opcode]
}

const fn pull<OP: ops::PullOp>() -> [MicroOp; 5] {
    [dummy_read, inc_sp_read_stack, pull_read, stack_pull::<OP>, fetch_opcode]
}

// ======================================================================
// Table builder
// ======================================================================

const fn build_steps() -> [MicroOp; TABLE_SIZE] {
    let mut t = [trap as MicroOp; TABLE_SIZE];

    // BRK ($00) — also handles IRQ, NMI, RESET via brk_flags.
    set(&mut t, 0x00, &[brk_t0, brk_push_pch, brk_push_pcl, brk_push_p,
                         brk_vector_lo, brk_read_vector_lo, latch_to_pc, fetch_opcode]);

    // --- Immediate ---
    set(&mut t, 0x69, &imm_read::<ops::Adc>());
    set(&mut t, 0xE9, &imm_read::<ops::Sbc>());
    set(&mut t, 0x29, &imm_read::<ops::And>());
    set(&mut t, 0x09, &imm_read::<ops::Ora>());
    set(&mut t, 0x49, &imm_read::<ops::Eor>());
    set(&mut t, 0xC9, &imm_read::<ops::Cmp>());
    set(&mut t, 0xE0, &imm_read::<ops::Cpx>());
    set(&mut t, 0xC0, &imm_read::<ops::Cpy>());
    set(&mut t, 0xA9, &imm_read::<ops::Lda>());
    set(&mut t, 0xA2, &imm_read::<ops::Ldx>());
    set(&mut t, 0xA0, &imm_read::<ops::Ldy>());

    // --- Zero page read ---
    set(&mut t, 0x65, &zp_read::<ops::Adc>());
    set(&mut t, 0xE5, &zp_read::<ops::Sbc>());
    set(&mut t, 0x25, &zp_read::<ops::And>());
    set(&mut t, 0x05, &zp_read::<ops::Ora>());
    set(&mut t, 0x45, &zp_read::<ops::Eor>());
    set(&mut t, 0xC5, &zp_read::<ops::Cmp>());
    set(&mut t, 0xE4, &zp_read::<ops::Cpx>());
    set(&mut t, 0xC4, &zp_read::<ops::Cpy>());
    set(&mut t, 0x24, &zp_read::<ops::Bit>());
    set(&mut t, 0xA5, &zp_read::<ops::Lda>());
    set(&mut t, 0xA6, &zp_read::<ops::Ldx>());
    set(&mut t, 0xA4, &zp_read::<ops::Ldy>());

    // --- Zero page write ---
    set(&mut t, 0x85, &zp_write::<ops::Sta>());
    set(&mut t, 0x86, &zp_write::<ops::Stx>());
    set(&mut t, 0x84, &zp_write::<ops::Sty>());

    // --- Zero page,X read ---
    set(&mut t, 0x75, &zp_x_read::<ops::Adc>());
    set(&mut t, 0xF5, &zp_x_read::<ops::Sbc>());
    set(&mut t, 0x35, &zp_x_read::<ops::And>());
    set(&mut t, 0x15, &zp_x_read::<ops::Ora>());
    set(&mut t, 0x55, &zp_x_read::<ops::Eor>());
    set(&mut t, 0xD5, &zp_x_read::<ops::Cmp>());
    set(&mut t, 0xB5, &zp_x_read::<ops::Lda>());
    set(&mut t, 0xB4, &zp_x_read::<ops::Ldy>());

    // --- Zero page,Y read ---
    set(&mut t, 0xB6, &zp_y_read::<ops::Ldx>());

    // --- Zero page,X write ---
    set(&mut t, 0x95, &zp_x_write::<ops::Sta>());
    set(&mut t, 0x94, &zp_x_write::<ops::Sty>());

    // --- Zero page,Y write ---
    set(&mut t, 0x96, &zp_y_write::<ops::Stx>());

    // --- Absolute read ---
    set(&mut t, 0x6D, &abs_read::<ops::Adc>());
    set(&mut t, 0xED, &abs_read::<ops::Sbc>());
    set(&mut t, 0x2D, &abs_read::<ops::And>());
    set(&mut t, 0x0D, &abs_read::<ops::Ora>());
    set(&mut t, 0x4D, &abs_read::<ops::Eor>());
    set(&mut t, 0xCD, &abs_read::<ops::Cmp>());
    set(&mut t, 0xEC, &abs_read::<ops::Cpx>());
    set(&mut t, 0xCC, &abs_read::<ops::Cpy>());
    set(&mut t, 0x2C, &abs_read::<ops::Bit>());
    set(&mut t, 0xAD, &abs_read::<ops::Lda>());
    set(&mut t, 0xAE, &abs_read::<ops::Ldx>());
    set(&mut t, 0xAC, &abs_read::<ops::Ldy>());

    // --- Absolute write ---
    set(&mut t, 0x8D, &abs_write::<ops::Sta>());
    set(&mut t, 0x8E, &abs_write::<ops::Stx>());
    set(&mut t, 0x8C, &abs_write::<ops::Sty>());

    // --- Absolute,X read ---
    set(&mut t, 0x7D, &abs_x_read::<ops::Adc>());
    set(&mut t, 0xFD, &abs_x_read::<ops::Sbc>());
    set(&mut t, 0x3D, &abs_x_read::<ops::And>());
    set(&mut t, 0x1D, &abs_x_read::<ops::Ora>());
    set(&mut t, 0x5D, &abs_x_read::<ops::Eor>());
    set(&mut t, 0xDD, &abs_x_read::<ops::Cmp>());
    set(&mut t, 0xBD, &abs_x_read::<ops::Lda>());
    set(&mut t, 0xBC, &abs_x_read::<ops::Ldy>());

    // --- Absolute,Y read ---
    set(&mut t, 0x79, &abs_y_read::<ops::Adc>());
    set(&mut t, 0xF9, &abs_y_read::<ops::Sbc>());
    set(&mut t, 0x39, &abs_y_read::<ops::And>());
    set(&mut t, 0x19, &abs_y_read::<ops::Ora>());
    set(&mut t, 0x59, &abs_y_read::<ops::Eor>());
    set(&mut t, 0xD9, &abs_y_read::<ops::Cmp>());
    set(&mut t, 0xB9, &abs_y_read::<ops::Lda>());
    set(&mut t, 0xBE, &abs_y_read::<ops::Ldx>());

    // --- Absolute,X write ---
    set(&mut t, 0x9D, &abs_x_write::<ops::Sta>());

    // --- Absolute,Y write ---
    set(&mut t, 0x99, &abs_y_write::<ops::Sta>());

    // --- (Indirect,X) read ---
    set(&mut t, 0x61, &ind_x_read::<ops::Adc>());
    set(&mut t, 0xE1, &ind_x_read::<ops::Sbc>());
    set(&mut t, 0x21, &ind_x_read::<ops::And>());
    set(&mut t, 0x01, &ind_x_read::<ops::Ora>());
    set(&mut t, 0x41, &ind_x_read::<ops::Eor>());
    set(&mut t, 0xC1, &ind_x_read::<ops::Cmp>());
    set(&mut t, 0xA1, &ind_x_read::<ops::Lda>());

    // --- (Indirect,X) write ---
    set(&mut t, 0x81, &ind_x_write::<ops::Sta>());

    // --- (Indirect),Y read ---
    set(&mut t, 0x71, &ind_y_read::<ops::Adc>());
    set(&mut t, 0xF1, &ind_y_read::<ops::Sbc>());
    set(&mut t, 0x31, &ind_y_read::<ops::And>());
    set(&mut t, 0x11, &ind_y_read::<ops::Ora>());
    set(&mut t, 0x51, &ind_y_read::<ops::Eor>());
    set(&mut t, 0xD1, &ind_y_read::<ops::Cmp>());
    set(&mut t, 0xB1, &ind_y_read::<ops::Lda>());

    // --- (Indirect),Y write ---
    set(&mut t, 0x91, &ind_y_write::<ops::Sta>());

    // --- Accumulator RMW ---
    set(&mut t, 0x0A, &acc_rmw::<ops::Asl>());
    set(&mut t, 0x4A, &acc_rmw::<ops::Lsr>());
    set(&mut t, 0x2A, &acc_rmw::<ops::Rol>());
    set(&mut t, 0x6A, &acc_rmw::<ops::Ror>());

    // --- Zero page RMW ---
    set(&mut t, 0x06, &zp_rmw::<ops::Asl>());
    set(&mut t, 0x46, &zp_rmw::<ops::Lsr>());
    set(&mut t, 0x26, &zp_rmw::<ops::Rol>());
    set(&mut t, 0x66, &zp_rmw::<ops::Ror>());
    set(&mut t, 0xE6, &zp_rmw::<ops::Inc>());
    set(&mut t, 0xC6, &zp_rmw::<ops::Dec>());

    // --- Zero page,X RMW ---
    set(&mut t, 0x16, &zp_x_rmw::<ops::Asl>());
    set(&mut t, 0x56, &zp_x_rmw::<ops::Lsr>());
    set(&mut t, 0x36, &zp_x_rmw::<ops::Rol>());
    set(&mut t, 0x76, &zp_x_rmw::<ops::Ror>());
    set(&mut t, 0xF6, &zp_x_rmw::<ops::Inc>());
    set(&mut t, 0xD6, &zp_x_rmw::<ops::Dec>());

    // --- Absolute RMW ---
    set(&mut t, 0x0E, &abs_rmw::<ops::Asl>());
    set(&mut t, 0x4E, &abs_rmw::<ops::Lsr>());
    set(&mut t, 0x2E, &abs_rmw::<ops::Rol>());
    set(&mut t, 0x6E, &abs_rmw::<ops::Ror>());
    set(&mut t, 0xEE, &abs_rmw::<ops::Inc>());
    set(&mut t, 0xCE, &abs_rmw::<ops::Dec>());

    // --- Absolute,X RMW ---
    set(&mut t, 0x1E, &abs_x_rmw::<ops::Asl>());
    set(&mut t, 0x5E, &abs_x_rmw::<ops::Lsr>());
    set(&mut t, 0x3E, &abs_x_rmw::<ops::Rol>());
    set(&mut t, 0x7E, &abs_x_rmw::<ops::Ror>());
    set(&mut t, 0xFE, &abs_x_rmw::<ops::Inc>());
    set(&mut t, 0xDE, &abs_x_rmw::<ops::Dec>());

    // --- Implied ---
    set(&mut t, 0xEA, &imp::<ops::Nop>());
    set(&mut t, 0xE8, &imp::<ops::Inx>());
    set(&mut t, 0xC8, &imp::<ops::Iny>());
    set(&mut t, 0xCA, &imp::<ops::Dex>());
    set(&mut t, 0x88, &imp::<ops::Dey>());
    set(&mut t, 0xAA, &imp::<ops::Tax>());
    set(&mut t, 0xA8, &imp::<ops::Tay>());
    set(&mut t, 0x8A, &imp::<ops::Txa>());
    set(&mut t, 0x98, &imp::<ops::Tya>());
    set(&mut t, 0xBA, &imp::<ops::Tsx>());
    set(&mut t, 0x9A, &imp::<ops::Txs>());
    set(&mut t, 0x18, &imp::<ops::Clc>());
    set(&mut t, 0x38, &imp::<ops::Sec>());
    set(&mut t, 0x58, &imp::<ops::Cli>());
    set(&mut t, 0x78, &imp::<ops::Sei>());
    set(&mut t, 0xB8, &imp::<ops::Clv>());
    set(&mut t, 0xD8, &imp::<ops::Cld>());
    set(&mut t, 0xF8, &imp::<ops::Sed>());

    // --- Branches ---
    use super::flags;
    set(&mut t, 0x90, &branch_op::<{flags::C}, false>());
    set(&mut t, 0xB0, &branch_op::<{flags::C}, true>());
    set(&mut t, 0xF0, &branch_op::<{flags::Z}, true>());
    set(&mut t, 0xD0, &branch_op::<{flags::Z}, false>());
    set(&mut t, 0x30, &branch_op::<{flags::N}, true>());
    set(&mut t, 0x10, &branch_op::<{flags::N}, false>());
    set(&mut t, 0x70, &branch_op::<{flags::V}, true>());
    set(&mut t, 0x50, &branch_op::<{flags::V}, false>());

    // --- JMP ---
    set(&mut t, 0x4C, &[fetch_addr_lo, latch_to_pc, fetch_opcode]);
    set(&mut t, 0x6C, &[fetch_addr_lo, latch_to_base_hi, jmp_ind_lo, latch_to_pc, fetch_opcode]);

    // --- JSR ---
    set(&mut t, 0x20, &[jsr_save_lo, jsr_push_pch, jsr_push_pcl, dummy_read, latch_to_pc, fetch_opcode]);

    // --- RTS ---
    set(&mut t, 0x60, &[dummy_read, inc_sp_read_stack, inc_sp_read_stack, latch_to_base_read_stack, rts_read_pch, fetch_opcode]);

    // --- RTI ---
    set(&mut t, 0x40, &[dummy_read, inc_sp_read_stack, inc_sp_read_stack, rti_read_p, latch_to_base_read_stack, latch_to_pc, fetch_opcode]);

    // --- Stack ---
    set(&mut t, 0x48, &push::<ops::Pha>());
    set(&mut t, 0x08, &push::<ops::Php>());
    set(&mut t, 0x68, &pull::<ops::Pla>());
    set(&mut t, 0x28, &pull::<ops::Plp>());

    t
}
