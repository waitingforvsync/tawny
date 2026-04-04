/// Micro-op step table for all 6502 instructions.
///
/// Indexed by (opcode << 3) | step. Each addressing mode is a const generic
/// function returning a fixed-size array of micro-ops, parameterised by
/// operation. The table entries are then one-liners.
use super::addr::*;
use super::ops;
use super::{MicroOp, TABLE_SIZE, MAX_STEPS};

pub static STEPS: [MicroOp; TABLE_SIZE] = build_steps();

fn trap(cpu: &mut super::Mos6502) -> super::Mos6502Output {
    cpu.tstate -= 1;
    super::read(cpu.pc)
}

/// Dispatch the micro-op for the current tstate.
#[inline(always)]
pub fn dispatch(cpu: &mut super::Mos6502) -> super::Mos6502Output {
    STEPS[cpu.tstate](cpu)
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
// Addressing mode step generators (const generic on operation)
// ======================================================================

// --- Read modes ---

const fn imm_read<const OP: u8>() -> [MicroOp; 2] {
    [fetch_operand, final_read::<OP>]
}

const fn zp_read<const OP: u8>() -> [MicroOp; 3] {
    [fetch_operand, addr_zp, final_read::<OP>]
}

const fn zp_x_read<const OP: u8>() -> [MicroOp; 4] {
    [fetch_operand, addr_zp_x, addr_zp_indexed, final_read::<OP>]
}

const fn zp_y_read<const OP: u8>() -> [MicroOp; 4] {
    [fetch_operand, addr_zp_y, addr_zp_indexed, final_read::<OP>]
}

const fn abs_read<const OP: u8>() -> [MicroOp; 4] {
    [fetch_operand, fetch_addr_hi, addr_abs, final_read::<OP>]
}

const fn abs_x_read<const OP: u8>() -> [MicroOp; 5] {
    [fetch_operand, fetch_addr_hi_add_x, addr_abs_indexed, addr_fixup, final_read::<OP>]
}

const fn abs_y_read<const OP: u8>() -> [MicroOp; 5] {
    [fetch_operand, fetch_addr_hi_add_y, addr_abs_indexed, addr_fixup, final_read::<OP>]
}

const fn ind_x_read<const OP: u8>() -> [MicroOp; 6] {
    [fetch_operand, addr_zp_x, addr_indirect_lo, addr_indirect_hi, addr_indirect_target, final_read::<OP>]
}

const fn ind_y_read<const OP: u8>() -> [MicroOp; 6] {
    [fetch_operand, addr_ind_y_lo, addr_ind_y_hi, addr_abs_indexed, addr_fixup, final_read::<OP>]
}

// --- Write modes ---

const fn zp_write<const OP: u8>() -> [MicroOp; 3] {
    [fetch_operand, write_zp::<OP>, fetch_opcode]
}

const fn zp_x_write<const OP: u8>() -> [MicroOp; 4] {
    [fetch_operand, addr_zp_x, write_zp_indexed::<OP>, fetch_opcode]
}

const fn zp_y_write<const OP: u8>() -> [MicroOp; 4] {
    [fetch_operand, addr_zp_y, write_zp_indexed::<OP>, fetch_opcode]
}

const fn abs_write<const OP: u8>() -> [MicroOp; 4] {
    [fetch_operand, fetch_addr_hi, write_abs::<OP>, fetch_opcode]
}

const fn abs_x_write<const OP: u8>() -> [MicroOp; 5] {
    [fetch_operand, fetch_addr_hi_add_x, addr_abs_indexed_penalty, write_fixup::<OP>, fetch_opcode]
}

const fn abs_y_write<const OP: u8>() -> [MicroOp; 5] {
    [fetch_operand, fetch_addr_hi_add_y, addr_abs_indexed_penalty, write_fixup::<OP>, fetch_opcode]
}

const fn ind_x_write<const OP: u8>() -> [MicroOp; 6] {
    [fetch_operand, addr_zp_x, addr_indirect_lo, addr_indirect_hi, write_indirect_target::<OP>, fetch_opcode]
}

const fn ind_y_write<const OP: u8>() -> [MicroOp; 6] {
    [fetch_operand, addr_ind_y_lo, addr_ind_y_hi, addr_abs_indexed_penalty, write_fixup::<OP>, fetch_opcode]
}

// --- RMW modes ---

const fn acc_rmw<const OP: u8>() -> [MicroOp; 2] {
    [accumulator::<OP>, fetch_opcode]
}

const fn zp_rmw<const OP: u8>() -> [MicroOp; 5] {
    [fetch_operand, addr_zp, rmw_modify::<OP>, rmw_write, fetch_opcode]
}

const fn zp_x_rmw<const OP: u8>() -> [MicroOp; 6] {
    [fetch_operand, addr_zp_x, addr_zp_indexed, rmw_modify::<OP>, rmw_write, fetch_opcode]
}

const fn abs_rmw<const OP: u8>() -> [MicroOp; 6] {
    [fetch_operand, fetch_addr_hi, addr_abs, rmw_modify::<OP>, rmw_write, fetch_opcode]
}

const fn abs_x_rmw<const OP: u8>() -> [MicroOp; 7] {
    [fetch_operand, fetch_addr_hi_add_x, addr_abs_indexed_penalty, addr_fixup, rmw_modify::<OP>, rmw_write, fetch_opcode]
}

// --- Implied ---

const fn imp<const OP: u8>() -> [MicroOp; 2] {
    [implied::<OP>, fetch_opcode]
}

// ======================================================================
// Table builder
// ======================================================================

const fn build_steps() -> [MicroOp; TABLE_SIZE] {
    let mut t = [trap as MicroOp; TABLE_SIZE];

    // BRK ($00) — 7 cycles. Handles IRQ, NMI, RESET via brk_flags.
    set(&mut t, 0x00, &[brk_t0, brk_t1, brk_t2, brk_t3, brk_t4, brk_t5, brk_t6]);

    // --- Immediate ---
    set(&mut t, 0x69, &imm_read::<{ops::ADC}>());
    set(&mut t, 0xE9, &imm_read::<{ops::SBC}>());
    set(&mut t, 0x29, &imm_read::<{ops::AND}>());
    set(&mut t, 0x09, &imm_read::<{ops::ORA}>());
    set(&mut t, 0x49, &imm_read::<{ops::EOR}>());
    set(&mut t, 0xC9, &imm_read::<{ops::CMP}>());
    set(&mut t, 0xE0, &imm_read::<{ops::CPX}>());
    set(&mut t, 0xC0, &imm_read::<{ops::CPY}>());
    set(&mut t, 0xA9, &imm_read::<{ops::LDA}>());
    set(&mut t, 0xA2, &imm_read::<{ops::LDX}>());
    set(&mut t, 0xA0, &imm_read::<{ops::LDY}>());

    // --- Zero page read ---
    set(&mut t, 0x65, &zp_read::<{ops::ADC}>());
    set(&mut t, 0xE5, &zp_read::<{ops::SBC}>());
    set(&mut t, 0x25, &zp_read::<{ops::AND}>());
    set(&mut t, 0x05, &zp_read::<{ops::ORA}>());
    set(&mut t, 0x45, &zp_read::<{ops::EOR}>());
    set(&mut t, 0xC5, &zp_read::<{ops::CMP}>());
    set(&mut t, 0xE4, &zp_read::<{ops::CPX}>());
    set(&mut t, 0xC4, &zp_read::<{ops::CPY}>());
    set(&mut t, 0x24, &zp_read::<{ops::BIT}>());
    set(&mut t, 0xA5, &zp_read::<{ops::LDA}>());
    set(&mut t, 0xA6, &zp_read::<{ops::LDX}>());
    set(&mut t, 0xA4, &zp_read::<{ops::LDY}>());

    // --- Zero page write ---
    set(&mut t, 0x85, &zp_write::<{ops::STA}>());
    set(&mut t, 0x86, &zp_write::<{ops::STX}>());
    set(&mut t, 0x84, &zp_write::<{ops::STY}>());

    // --- Zero page,X read ---
    set(&mut t, 0x75, &zp_x_read::<{ops::ADC}>());
    set(&mut t, 0xF5, &zp_x_read::<{ops::SBC}>());
    set(&mut t, 0x35, &zp_x_read::<{ops::AND}>());
    set(&mut t, 0x15, &zp_x_read::<{ops::ORA}>());
    set(&mut t, 0x55, &zp_x_read::<{ops::EOR}>());
    set(&mut t, 0xD5, &zp_x_read::<{ops::CMP}>());
    set(&mut t, 0xB5, &zp_x_read::<{ops::LDA}>());
    set(&mut t, 0xB4, &zp_x_read::<{ops::LDY}>());

    // --- Zero page,Y read ---
    set(&mut t, 0xB6, &zp_y_read::<{ops::LDX}>());

    // --- Zero page,X write ---
    set(&mut t, 0x95, &zp_x_write::<{ops::STA}>());
    set(&mut t, 0x94, &zp_x_write::<{ops::STY}>());

    // --- Zero page,Y write ---
    set(&mut t, 0x96, &zp_y_write::<{ops::STX}>());

    // --- Absolute read ---
    set(&mut t, 0x6D, &abs_read::<{ops::ADC}>());
    set(&mut t, 0xED, &abs_read::<{ops::SBC}>());
    set(&mut t, 0x2D, &abs_read::<{ops::AND}>());
    set(&mut t, 0x0D, &abs_read::<{ops::ORA}>());
    set(&mut t, 0x4D, &abs_read::<{ops::EOR}>());
    set(&mut t, 0xCD, &abs_read::<{ops::CMP}>());
    set(&mut t, 0xEC, &abs_read::<{ops::CPX}>());
    set(&mut t, 0xCC, &abs_read::<{ops::CPY}>());
    set(&mut t, 0x2C, &abs_read::<{ops::BIT}>());
    set(&mut t, 0xAD, &abs_read::<{ops::LDA}>());
    set(&mut t, 0xAE, &abs_read::<{ops::LDX}>());
    set(&mut t, 0xAC, &abs_read::<{ops::LDY}>());

    // --- Absolute write ---
    set(&mut t, 0x8D, &abs_write::<{ops::STA}>());
    set(&mut t, 0x8E, &abs_write::<{ops::STX}>());
    set(&mut t, 0x8C, &abs_write::<{ops::STY}>());

    // --- Absolute,X read ---
    set(&mut t, 0x7D, &abs_x_read::<{ops::ADC}>());
    set(&mut t, 0xFD, &abs_x_read::<{ops::SBC}>());
    set(&mut t, 0x3D, &abs_x_read::<{ops::AND}>());
    set(&mut t, 0x1D, &abs_x_read::<{ops::ORA}>());
    set(&mut t, 0x5D, &abs_x_read::<{ops::EOR}>());
    set(&mut t, 0xDD, &abs_x_read::<{ops::CMP}>());
    set(&mut t, 0xBD, &abs_x_read::<{ops::LDA}>());
    set(&mut t, 0xBC, &abs_x_read::<{ops::LDY}>());

    // --- Absolute,Y read ---
    set(&mut t, 0x79, &abs_y_read::<{ops::ADC}>());
    set(&mut t, 0xF9, &abs_y_read::<{ops::SBC}>());
    set(&mut t, 0x39, &abs_y_read::<{ops::AND}>());
    set(&mut t, 0x19, &abs_y_read::<{ops::ORA}>());
    set(&mut t, 0x59, &abs_y_read::<{ops::EOR}>());
    set(&mut t, 0xD9, &abs_y_read::<{ops::CMP}>());
    set(&mut t, 0xB9, &abs_y_read::<{ops::LDA}>());
    set(&mut t, 0xBE, &abs_y_read::<{ops::LDX}>());

    // --- Absolute,X write ---
    set(&mut t, 0x9D, &abs_x_write::<{ops::STA}>());

    // --- Absolute,Y write ---
    set(&mut t, 0x99, &abs_y_write::<{ops::STA}>());

    // --- (Indirect,X) read ---
    set(&mut t, 0x61, &ind_x_read::<{ops::ADC}>());
    set(&mut t, 0xE1, &ind_x_read::<{ops::SBC}>());
    set(&mut t, 0x21, &ind_x_read::<{ops::AND}>());
    set(&mut t, 0x01, &ind_x_read::<{ops::ORA}>());
    set(&mut t, 0x41, &ind_x_read::<{ops::EOR}>());
    set(&mut t, 0xC1, &ind_x_read::<{ops::CMP}>());
    set(&mut t, 0xA1, &ind_x_read::<{ops::LDA}>());

    // --- (Indirect,X) write ---
    set(&mut t, 0x81, &ind_x_write::<{ops::STA}>());

    // --- (Indirect),Y read ---
    set(&mut t, 0x71, &ind_y_read::<{ops::ADC}>());
    set(&mut t, 0xF1, &ind_y_read::<{ops::SBC}>());
    set(&mut t, 0x31, &ind_y_read::<{ops::AND}>());
    set(&mut t, 0x11, &ind_y_read::<{ops::ORA}>());
    set(&mut t, 0x51, &ind_y_read::<{ops::EOR}>());
    set(&mut t, 0xD1, &ind_y_read::<{ops::CMP}>());
    set(&mut t, 0xB1, &ind_y_read::<{ops::LDA}>());

    // --- (Indirect),Y write ---
    set(&mut t, 0x91, &ind_y_write::<{ops::STA}>());

    // --- Accumulator RMW ---
    set(&mut t, 0x0A, &acc_rmw::<{ops::ASL}>());
    set(&mut t, 0x4A, &acc_rmw::<{ops::LSR}>());
    set(&mut t, 0x2A, &acc_rmw::<{ops::ROL}>());
    set(&mut t, 0x6A, &acc_rmw::<{ops::ROR}>());

    // --- Zero page RMW ---
    set(&mut t, 0x06, &zp_rmw::<{ops::ASL}>());
    set(&mut t, 0x46, &zp_rmw::<{ops::LSR}>());
    set(&mut t, 0x26, &zp_rmw::<{ops::ROL}>());
    set(&mut t, 0x66, &zp_rmw::<{ops::ROR}>());
    set(&mut t, 0xE6, &zp_rmw::<{ops::INC}>());
    set(&mut t, 0xC6, &zp_rmw::<{ops::DEC}>());

    // --- Zero page,X RMW ---
    set(&mut t, 0x16, &zp_x_rmw::<{ops::ASL}>());
    set(&mut t, 0x56, &zp_x_rmw::<{ops::LSR}>());
    set(&mut t, 0x36, &zp_x_rmw::<{ops::ROL}>());
    set(&mut t, 0x76, &zp_x_rmw::<{ops::ROR}>());
    set(&mut t, 0xF6, &zp_x_rmw::<{ops::INC}>());
    set(&mut t, 0xD6, &zp_x_rmw::<{ops::DEC}>());

    // --- Absolute RMW ---
    set(&mut t, 0x0E, &abs_rmw::<{ops::ASL}>());
    set(&mut t, 0x4E, &abs_rmw::<{ops::LSR}>());
    set(&mut t, 0x2E, &abs_rmw::<{ops::ROL}>());
    set(&mut t, 0x6E, &abs_rmw::<{ops::ROR}>());
    set(&mut t, 0xEE, &abs_rmw::<{ops::INC}>());
    set(&mut t, 0xCE, &abs_rmw::<{ops::DEC}>());

    // --- Absolute,X RMW ---
    set(&mut t, 0x1E, &abs_x_rmw::<{ops::ASL}>());
    set(&mut t, 0x5E, &abs_x_rmw::<{ops::LSR}>());
    set(&mut t, 0x3E, &abs_x_rmw::<{ops::ROL}>());
    set(&mut t, 0x7E, &abs_x_rmw::<{ops::ROR}>());
    set(&mut t, 0xFE, &abs_x_rmw::<{ops::INC}>());
    set(&mut t, 0xDE, &abs_x_rmw::<{ops::DEC}>());

    // --- Implied ---
    set(&mut t, 0xEA, &imp::<{ops::NOP}>());
    set(&mut t, 0xE8, &imp::<{ops::INX}>());
    set(&mut t, 0xC8, &imp::<{ops::INY}>());
    set(&mut t, 0xCA, &imp::<{ops::DEX}>());
    set(&mut t, 0x88, &imp::<{ops::DEY}>());
    set(&mut t, 0xAA, &imp::<{ops::TAX}>());
    set(&mut t, 0xA8, &imp::<{ops::TAY}>());
    set(&mut t, 0x8A, &imp::<{ops::TXA}>());
    set(&mut t, 0x98, &imp::<{ops::TYA}>());
    set(&mut t, 0xBA, &imp::<{ops::TSX}>());
    set(&mut t, 0x9A, &imp::<{ops::TXS}>());
    set(&mut t, 0x18, &imp::<{ops::CLC}>());
    set(&mut t, 0x38, &imp::<{ops::SEC}>());
    set(&mut t, 0x58, &imp::<{ops::CLI}>());
    set(&mut t, 0x78, &imp::<{ops::SEI}>());
    set(&mut t, 0xB8, &imp::<{ops::CLV}>());
    set(&mut t, 0xD8, &imp::<{ops::CLD}>());
    set(&mut t, 0xF8, &imp::<{ops::SED}>());

    // --- Branches (unique per flag/sense, not templated on Op) ---
    use super::flags;
    set(&mut t, 0x90, &[fetch_operand, branch::<{flags::C}, false>, branch_take, branch_fixup]);
    set(&mut t, 0xB0, &[fetch_operand, branch::<{flags::C}, true>,  branch_take, branch_fixup]);
    set(&mut t, 0xF0, &[fetch_operand, branch::<{flags::Z}, true>,  branch_take, branch_fixup]);
    set(&mut t, 0xD0, &[fetch_operand, branch::<{flags::Z}, false>, branch_take, branch_fixup]);
    set(&mut t, 0x30, &[fetch_operand, branch::<{flags::N}, true>,  branch_take, branch_fixup]);
    set(&mut t, 0x10, &[fetch_operand, branch::<{flags::N}, false>, branch_take, branch_fixup]);
    set(&mut t, 0x70, &[fetch_operand, branch::<{flags::V}, true>,  branch_take, branch_fixup]);
    set(&mut t, 0x50, &[fetch_operand, branch::<{flags::V}, false>, branch_take, branch_fixup]);

    // --- JMP ---
    set(&mut t, 0x4C, &[fetch_operand, fetch_addr_hi, jmp_abs]);
    set(&mut t, 0x6C, &[fetch_operand, fetch_addr_hi, addr_jmp_indirect, addr_jmp_indirect_hi, jmp_indirect_done]);

    // --- JSR / RTS / RTI ---
    set(&mut t, 0x20, &[fetch_operand, jsr_save_lo, jsr_push_pch, jsr_push_pcl, jsr_fetch_hi, jsr_done]);
    set(&mut t, 0x60, &[rts_dummy, rts_inc_sp, rts_addr_pcl, rts_addr_pch, rts_done, fetch_opcode]);
    set(&mut t, 0x40, &[rti_dummy, rti_inc_sp, rti_addr_p, rti_addr_pcl, rti_addr_pch, rti_done]);

    // --- Stack ---
    set(&mut t, 0x48, &[push_dummy, pha_push, fetch_opcode]);
    set(&mut t, 0x08, &[push_dummy, php_push, fetch_opcode]);
    set(&mut t, 0x68, &[pull_dummy, pull_inc_sp, addr_pull, pla_done]);
    set(&mut t, 0x28, &[pull_dummy, pull_inc_sp, addr_pull, plp_done]);

    t
}
