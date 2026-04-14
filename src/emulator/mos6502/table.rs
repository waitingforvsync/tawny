/// Micro-op step table and disassembly table for all 6502 instructions.
///
/// Every instruction ends with `fetch_opcode`, which consumes the opcode
/// byte from data_latch, decodes it, and sets tstate for the next instruction.
///
/// Each addressing mode generator returns an `OpSteps` containing both the
/// micro-op steps and the disassembly metadata (mnemonic + addressing mode).
use super::addr::*;
use super::ops;
use super::{MicroOp, Mnemonic, AddrMode, OpEntry, TABLE_SIZE, MAX_STEPS};

use Mnemonic as M;
use AddrMode as A;

struct Tables {
    steps: [MicroOp; TABLE_SIZE],
    disasm: [OpEntry; 256],
}

static TABLES: Tables = build_tables();

pub static STEPS: &[MicroOp; TABLE_SIZE] = &TABLES.steps;
pub static DISASM: &[OpEntry; 256] = &TABLES.disasm;

#[inline(always)]
pub fn dispatch(cpu: &mut super::Mos6502, data: u8) -> super::Mos6502Output {
    TABLES.steps[cpu.tstate as usize](cpu, data)
}

// ======================================================================
// OpSteps: micro-op steps + disassembly entry
// ======================================================================

struct OpSteps<const N: usize> {
    steps: [MicroOp; N],
    entry: OpEntry,
}

const fn set<const N: usize>(tables: &mut Tables, opcode: usize, op: &OpSteps<N>) {
    let base = opcode * MAX_STEPS;
    let mut i = 0;
    while i < N {
        tables.steps[base + i] = op.steps[i];
        i += 1;
    }
    tables.disasm[opcode] = op.entry;
}

// ======================================================================
// Addressing mode generators
// ======================================================================

const fn imm_read<OP: ops::ReadOp>() -> OpSteps<2> {
    OpSteps {
        steps: [fetch_data::<OP>, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::Immediate),
    }
}

const fn zp_read<OP: ops::ReadOp>() -> OpSteps<3> {
    OpSteps {
        steps: [read_zp, fetch_data::<OP>, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::ZeroPage),
    }
}

const fn zp_write<OP: ops::StoreOp>() -> OpSteps<3> {
    OpSteps {
        steps: [write_zp::<OP>, opcode_read, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::ZeroPage),
    }
}

const fn zp_x_read<OP: ops::ReadOp>() -> OpSteps<4> {
    OpSteps {
        steps: [latch_to_base, add_index_x, fetch_data::<OP>, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::ZeroPageX),
    }
}

const fn zp_y_read<OP: ops::ReadOp>() -> OpSteps<4> {
    OpSteps {
        steps: [latch_to_base, add_index_y, fetch_data::<OP>, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::ZeroPageY),
    }
}

const fn zp_x_write<OP: ops::StoreOp>() -> OpSteps<4> {
    OpSteps {
        steps: [index_zp_x, write_base::<OP>, opcode_read, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::ZeroPageX),
    }
}

const fn zp_y_write<OP: ops::StoreOp>() -> OpSteps<4> {
    OpSteps {
        steps: [index_zp_y, write_base::<OP>, opcode_read, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::ZeroPageY),
    }
}

const fn abs_read<OP: ops::ReadOp>() -> OpSteps<4> {
    OpSteps {
        steps: [fetch_addr_lo, read_base_hi, fetch_data::<OP>, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::Absolute),
    }
}

const fn abs_write<OP: ops::StoreOp>() -> OpSteps<4> {
    OpSteps {
        steps: [fetch_addr_lo, write_abs::<OP>, opcode_read, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::Absolute),
    }
}

const fn abs_x_read<OP: ops::ReadOp>() -> OpSteps<5> {
    OpSteps {
        steps: [fetch_addr_lo_x, fetch_addr_hi_indexed, read_base, fetch_data::<OP>, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::AbsoluteX),
    }
}

const fn abs_y_read<OP: ops::ReadOp>() -> OpSteps<5> {
    OpSteps {
        steps: [fetch_addr_lo_y, fetch_addr_hi_indexed, read_base, fetch_data::<OP>, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::AbsoluteY),
    }
}

const fn abs_x_write<OP: ops::StoreOp>() -> OpSteps<5> {
    OpSteps {
        steps: [fetch_addr_lo_x, fetch_addr_hi_indexed_penalty, write_base::<OP>, opcode_read, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::AbsoluteX),
    }
}

const fn abs_y_write<OP: ops::StoreOp>() -> OpSteps<5> {
    OpSteps {
        steps: [fetch_addr_lo_y, fetch_addr_hi_indexed_penalty, write_base::<OP>, opcode_read, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::AbsoluteY),
    }
}

const fn ind_x_read<OP: ops::ReadOp>() -> OpSteps<6> {
    OpSteps {
        steps: [index_zp_x, read_base, fetch_ind_lo, read_base_hi, fetch_data::<OP>, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::IndirectX),
    }
}

const fn ind_x_write<OP: ops::StoreOp>() -> OpSteps<6> {
    OpSteps {
        steps: [index_zp_x, read_base, fetch_ind_lo, write_abs::<OP>, opcode_read, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::IndirectX),
    }
}

const fn ind_y_read<OP: ops::ReadOp>() -> OpSteps<6> {
    OpSteps {
        steps: [latch_to_base, fetch_ind_y_lo, fetch_addr_hi_indexed, read_base, fetch_data::<OP>, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::IndirectY),
    }
}

const fn ind_y_write<OP: ops::StoreOp>() -> OpSteps<6> {
    OpSteps {
        steps: [latch_to_base, fetch_ind_y_lo, fetch_addr_hi_indexed_penalty, write_base::<OP>, opcode_read, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::IndirectY),
    }
}

const fn acc_rmw<OP: ops::RmwOp>() -> OpSteps<2> {
    OpSteps {
        steps: [accumulator::<OP>, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::Accumulator),
    }
}

const fn zp_rmw<OP: ops::RmwOp>() -> OpSteps<5> {
    OpSteps {
        steps: [latch_to_base, rmw_dummy_write, rmw_execute::<OP>, opcode_read, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::ZeroPage),
    }
}

const fn zp_x_rmw<OP: ops::RmwOp>() -> OpSteps<6> {
    OpSteps {
        steps: [index_zp_x, read_base, rmw_dummy_write, rmw_execute::<OP>, opcode_read, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::ZeroPageX),
    }
}

const fn abs_rmw<OP: ops::RmwOp>() -> OpSteps<6> {
    OpSteps {
        steps: [fetch_addr_lo, latch_to_base_hi, rmw_dummy_write, rmw_execute::<OP>, opcode_read, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::Absolute),
    }
}

const fn abs_x_rmw<OP: ops::RmwOp>() -> OpSteps<7> {
    OpSteps {
        steps: [fetch_addr_lo_x, fetch_addr_hi_indexed_penalty, read_base, rmw_dummy_write, rmw_execute::<OP>, opcode_read, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::AbsoluteX),
    }
}

const fn imp<OP: ops::ImpliedOp>() -> OpSteps<2> {
    OpSteps {
        steps: [implied::<OP>, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::Implied),
    }
}

const fn branch_op<const FLAG: u8, const SET: bool>(mnemonic: Mnemonic) -> OpSteps<4> {
    OpSteps {
        steps: [branch::<FLAG, SET>, branch_take, opcode_read, fetch_opcode],
        entry: OpEntry::new(mnemonic, A::Relative),
    }
}

const fn push<OP: ops::PushOp>() -> OpSteps<3> {
    OpSteps {
        steps: [stack_push::<OP>, opcode_read, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::Implied),
    }
}

const fn pull<OP: ops::PullOp>() -> OpSteps<4> {
    OpSteps {
        steps: [inc_sp_read_stack, pull_read, stack_pull::<OP>, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::Implied),
    }
}

const fn brk() -> OpSteps<7> {
    OpSteps {
        steps: [brk_push_pch, brk_push_pcl, brk_push_p,
                brk_vector_lo, brk_read_vector_lo, latch_to_pc, fetch_opcode],
        entry: OpEntry::new(M::Brk, A::Implied),
    }
}

const fn jmp_abs() -> OpSteps<3> {
    OpSteps {
        steps: [fetch_addr_lo, latch_to_pc, fetch_opcode],
        entry: OpEntry::new(M::Jmp, A::Absolute),
    }
}

const fn jmp_ind() -> OpSteps<5> {
    OpSteps {
        steps: [fetch_addr_lo, latch_to_base_hi, jmp_ind_lo, latch_to_pc, fetch_opcode],
        entry: OpEntry::new(M::Jmp, A::Indirect),
    }
}

const fn jsr() -> OpSteps<6> {
    OpSteps {
        steps: [jsr_save_lo, jsr_push_pch, jsr_push_pcl, dummy_read, latch_to_pc, fetch_opcode],
        entry: OpEntry::new(M::Jsr, A::Absolute),
    }
}

const fn rts() -> OpSteps<6> {
    OpSteps {
        steps: [dummy_read, inc_sp_read_stack, inc_sp_read_stack,
                latch_to_base_read_stack, rts_read_pch, fetch_opcode],
        entry: OpEntry::new(M::Rts, A::Implied),
    }
}

const fn rti() -> OpSteps<6> {
    OpSteps {
        steps: [inc_sp_read_stack, inc_sp_read_stack, rti_read_p,
                latch_to_base_read_stack, latch_to_pc, fetch_opcode],
        entry: OpEntry::new(M::Rti, A::Implied),
    }
}

// ======================================================================
// Illegal addressing mode generators
// ======================================================================

const fn abs_y_rmw<OP: ops::RmwOp>() -> OpSteps<7> {
    OpSteps {
        steps: [fetch_addr_lo_y, fetch_addr_hi_indexed_penalty, read_base,
                rmw_dummy_write, rmw_execute::<OP>, opcode_read, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::AbsoluteY),
    }
}

const fn ind_x_rmw<OP: ops::RmwOp>() -> OpSteps<8> {
    OpSteps {
        steps: [index_zp_x, read_base, fetch_ind_lo, latch_to_base_hi,
                rmw_dummy_write, rmw_execute::<OP>, opcode_read, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::IndirectX),
    }
}

const fn ind_y_rmw<OP: ops::RmwOp>() -> OpSteps<8> {
    OpSteps {
        steps: [latch_to_base, fetch_ind_y_lo, fetch_addr_hi_indexed_penalty, read_base,
                rmw_dummy_write, rmw_execute::<OP>, opcode_read, fetch_opcode],
        entry: OpEntry::new(OP::MNEMONIC, A::IndirectY),
    }
}

const fn jam_op() -> OpSteps<1> {
    OpSteps {
        steps: [jam],
        entry: OpEntry::new(M::Jam, A::Implied),
    }
}

// NOP variants that read and discard — reuse existing read generators with Nrd.
// abs_x_read with Nrd gives the correct 4+1 cycle NOP abs,X.

// TAS ($9B): abs,Y write but with SP = A & X before store.
const fn tas_op() -> OpSteps<5> {
    OpSteps {
        steps: [fetch_addr_lo_y, fetch_addr_hi_indexed_penalty, tas_write, opcode_read, fetch_opcode],
        entry: OpEntry::new(M::Tas, A::AbsoluteY),
    }
}

// ======================================================================
// Table builder
// ======================================================================

const fn build_tables() -> Tables {
    let mut t = Tables {
        steps: [jam as MicroOp; TABLE_SIZE],
        disasm: [OpEntry::new(M::Nop, A::Implied); 256],
    };

    // === Legal opcodes ===

    // BRK ($00) — also handles IRQ, NMI, RESET via brk_flags.
    set(&mut t, 0x00, &brk());

    // Immediate
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

    // Zero page read
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

    // Zero page write
    set(&mut t, 0x85, &zp_write::<ops::Sta>());
    set(&mut t, 0x86, &zp_write::<ops::Stx>());
    set(&mut t, 0x84, &zp_write::<ops::Sty>());

    // Zero page,X read
    set(&mut t, 0x75, &zp_x_read::<ops::Adc>());
    set(&mut t, 0xF5, &zp_x_read::<ops::Sbc>());
    set(&mut t, 0x35, &zp_x_read::<ops::And>());
    set(&mut t, 0x15, &zp_x_read::<ops::Ora>());
    set(&mut t, 0x55, &zp_x_read::<ops::Eor>());
    set(&mut t, 0xD5, &zp_x_read::<ops::Cmp>());
    set(&mut t, 0xB5, &zp_x_read::<ops::Lda>());
    set(&mut t, 0xB4, &zp_x_read::<ops::Ldy>());

    // Zero page,Y read
    set(&mut t, 0xB6, &zp_y_read::<ops::Ldx>());

    // Zero page,X write
    set(&mut t, 0x95, &zp_x_write::<ops::Sta>());
    set(&mut t, 0x94, &zp_x_write::<ops::Sty>());

    // Zero page,Y write
    set(&mut t, 0x96, &zp_y_write::<ops::Stx>());

    // Absolute read
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

    // Absolute write
    set(&mut t, 0x8D, &abs_write::<ops::Sta>());
    set(&mut t, 0x8E, &abs_write::<ops::Stx>());
    set(&mut t, 0x8C, &abs_write::<ops::Sty>());

    // Absolute,X read
    set(&mut t, 0x7D, &abs_x_read::<ops::Adc>());
    set(&mut t, 0xFD, &abs_x_read::<ops::Sbc>());
    set(&mut t, 0x3D, &abs_x_read::<ops::And>());
    set(&mut t, 0x1D, &abs_x_read::<ops::Ora>());
    set(&mut t, 0x5D, &abs_x_read::<ops::Eor>());
    set(&mut t, 0xDD, &abs_x_read::<ops::Cmp>());
    set(&mut t, 0xBD, &abs_x_read::<ops::Lda>());
    set(&mut t, 0xBC, &abs_x_read::<ops::Ldy>());

    // Absolute,Y read
    set(&mut t, 0x79, &abs_y_read::<ops::Adc>());
    set(&mut t, 0xF9, &abs_y_read::<ops::Sbc>());
    set(&mut t, 0x39, &abs_y_read::<ops::And>());
    set(&mut t, 0x19, &abs_y_read::<ops::Ora>());
    set(&mut t, 0x59, &abs_y_read::<ops::Eor>());
    set(&mut t, 0xD9, &abs_y_read::<ops::Cmp>());
    set(&mut t, 0xB9, &abs_y_read::<ops::Lda>());
    set(&mut t, 0xBE, &abs_y_read::<ops::Ldx>());

    // Absolute,X write
    set(&mut t, 0x9D, &abs_x_write::<ops::Sta>());

    // Absolute,Y write
    set(&mut t, 0x99, &abs_y_write::<ops::Sta>());

    // (Indirect,X) read
    set(&mut t, 0x61, &ind_x_read::<ops::Adc>());
    set(&mut t, 0xE1, &ind_x_read::<ops::Sbc>());
    set(&mut t, 0x21, &ind_x_read::<ops::And>());
    set(&mut t, 0x01, &ind_x_read::<ops::Ora>());
    set(&mut t, 0x41, &ind_x_read::<ops::Eor>());
    set(&mut t, 0xC1, &ind_x_read::<ops::Cmp>());
    set(&mut t, 0xA1, &ind_x_read::<ops::Lda>());

    // (Indirect,X) write
    set(&mut t, 0x81, &ind_x_write::<ops::Sta>());

    // (Indirect),Y read
    set(&mut t, 0x71, &ind_y_read::<ops::Adc>());
    set(&mut t, 0xF1, &ind_y_read::<ops::Sbc>());
    set(&mut t, 0x31, &ind_y_read::<ops::And>());
    set(&mut t, 0x11, &ind_y_read::<ops::Ora>());
    set(&mut t, 0x51, &ind_y_read::<ops::Eor>());
    set(&mut t, 0xD1, &ind_y_read::<ops::Cmp>());
    set(&mut t, 0xB1, &ind_y_read::<ops::Lda>());

    // (Indirect),Y write
    set(&mut t, 0x91, &ind_y_write::<ops::Sta>());

    // Accumulator RMW
    set(&mut t, 0x0A, &acc_rmw::<ops::Asl>());
    set(&mut t, 0x4A, &acc_rmw::<ops::Lsr>());
    set(&mut t, 0x2A, &acc_rmw::<ops::Rol>());
    set(&mut t, 0x6A, &acc_rmw::<ops::Ror>());

    // Zero page RMW
    set(&mut t, 0x06, &zp_rmw::<ops::Asl>());
    set(&mut t, 0x46, &zp_rmw::<ops::Lsr>());
    set(&mut t, 0x26, &zp_rmw::<ops::Rol>());
    set(&mut t, 0x66, &zp_rmw::<ops::Ror>());
    set(&mut t, 0xE6, &zp_rmw::<ops::Inc>());
    set(&mut t, 0xC6, &zp_rmw::<ops::Dec>());

    // Zero page,X RMW
    set(&mut t, 0x16, &zp_x_rmw::<ops::Asl>());
    set(&mut t, 0x56, &zp_x_rmw::<ops::Lsr>());
    set(&mut t, 0x36, &zp_x_rmw::<ops::Rol>());
    set(&mut t, 0x76, &zp_x_rmw::<ops::Ror>());
    set(&mut t, 0xF6, &zp_x_rmw::<ops::Inc>());
    set(&mut t, 0xD6, &zp_x_rmw::<ops::Dec>());

    // Absolute RMW
    set(&mut t, 0x0E, &abs_rmw::<ops::Asl>());
    set(&mut t, 0x4E, &abs_rmw::<ops::Lsr>());
    set(&mut t, 0x2E, &abs_rmw::<ops::Rol>());
    set(&mut t, 0x6E, &abs_rmw::<ops::Ror>());
    set(&mut t, 0xEE, &abs_rmw::<ops::Inc>());
    set(&mut t, 0xCE, &abs_rmw::<ops::Dec>());

    // Absolute,X RMW
    set(&mut t, 0x1E, &abs_x_rmw::<ops::Asl>());
    set(&mut t, 0x5E, &abs_x_rmw::<ops::Lsr>());
    set(&mut t, 0x3E, &abs_x_rmw::<ops::Rol>());
    set(&mut t, 0x7E, &abs_x_rmw::<ops::Ror>());
    set(&mut t, 0xFE, &abs_x_rmw::<ops::Inc>());
    set(&mut t, 0xDE, &abs_x_rmw::<ops::Dec>());

    // Implied
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

    // Branches
    use super::flags;
    set(&mut t, 0x90, &branch_op::<{flags::C}, false>(M::Bcc));
    set(&mut t, 0xB0, &branch_op::<{flags::C}, true>(M::Bcs));
    set(&mut t, 0xF0, &branch_op::<{flags::Z}, true>(M::Beq));
    set(&mut t, 0xD0, &branch_op::<{flags::Z}, false>(M::Bne));
    set(&mut t, 0x30, &branch_op::<{flags::N}, true>(M::Bmi));
    set(&mut t, 0x10, &branch_op::<{flags::N}, false>(M::Bpl));
    set(&mut t, 0x70, &branch_op::<{flags::V}, true>(M::Bvs));
    set(&mut t, 0x50, &branch_op::<{flags::V}, false>(M::Bvc));

    // JMP
    set(&mut t, 0x4C, &jmp_abs());
    set(&mut t, 0x6C, &jmp_ind());

    // JSR / RTS / RTI
    set(&mut t, 0x20, &jsr());
    set(&mut t, 0x60, &rts());
    set(&mut t, 0x40, &rti());

    // Stack
    set(&mut t, 0x48, &push::<ops::Pha>());
    set(&mut t, 0x08, &push::<ops::Php>());
    set(&mut t, 0x68, &pull::<ops::Pla>());
    set(&mut t, 0x28, &pull::<ops::Plp>());

    // === Illegal opcodes ===

    // JAM — CPU halt
    set(&mut t, 0x02, &jam_op());
    set(&mut t, 0x12, &jam_op());
    set(&mut t, 0x22, &jam_op());
    set(&mut t, 0x32, &jam_op());
    set(&mut t, 0x42, &jam_op());
    set(&mut t, 0x52, &jam_op());
    set(&mut t, 0x62, &jam_op());
    set(&mut t, 0x72, &jam_op());
    set(&mut t, 0x92, &jam_op());
    set(&mut t, 0xB2, &jam_op());
    set(&mut t, 0xD2, &jam_op());
    set(&mut t, 0xF2, &jam_op());

    // SLO — ASL + ORA
    set(&mut t, 0x03, &ind_x_rmw::<ops::Slo>());
    set(&mut t, 0x07, &zp_rmw::<ops::Slo>());
    set(&mut t, 0x0F, &abs_rmw::<ops::Slo>());
    set(&mut t, 0x13, &ind_y_rmw::<ops::Slo>());
    set(&mut t, 0x17, &zp_x_rmw::<ops::Slo>());
    set(&mut t, 0x1B, &abs_y_rmw::<ops::Slo>());
    set(&mut t, 0x1F, &abs_x_rmw::<ops::Slo>());

    // RLA — ROL + AND
    set(&mut t, 0x23, &ind_x_rmw::<ops::Rla>());
    set(&mut t, 0x27, &zp_rmw::<ops::Rla>());
    set(&mut t, 0x2F, &abs_rmw::<ops::Rla>());
    set(&mut t, 0x33, &ind_y_rmw::<ops::Rla>());
    set(&mut t, 0x37, &zp_x_rmw::<ops::Rla>());
    set(&mut t, 0x3B, &abs_y_rmw::<ops::Rla>());
    set(&mut t, 0x3F, &abs_x_rmw::<ops::Rla>());

    // SRE — LSR + EOR
    set(&mut t, 0x43, &ind_x_rmw::<ops::Sre>());
    set(&mut t, 0x47, &zp_rmw::<ops::Sre>());
    set(&mut t, 0x4F, &abs_rmw::<ops::Sre>());
    set(&mut t, 0x53, &ind_y_rmw::<ops::Sre>());
    set(&mut t, 0x57, &zp_x_rmw::<ops::Sre>());
    set(&mut t, 0x5B, &abs_y_rmw::<ops::Sre>());
    set(&mut t, 0x5F, &abs_x_rmw::<ops::Sre>());

    // RRA — ROR + ADC
    set(&mut t, 0x63, &ind_x_rmw::<ops::Rra>());
    set(&mut t, 0x67, &zp_rmw::<ops::Rra>());
    set(&mut t, 0x6F, &abs_rmw::<ops::Rra>());
    set(&mut t, 0x73, &ind_y_rmw::<ops::Rra>());
    set(&mut t, 0x77, &zp_x_rmw::<ops::Rra>());
    set(&mut t, 0x7B, &abs_y_rmw::<ops::Rra>());
    set(&mut t, 0x7F, &abs_x_rmw::<ops::Rra>());

    // DCP — DEC + CMP
    set(&mut t, 0xC3, &ind_x_rmw::<ops::Dcp>());
    set(&mut t, 0xC7, &zp_rmw::<ops::Dcp>());
    set(&mut t, 0xCF, &abs_rmw::<ops::Dcp>());
    set(&mut t, 0xD3, &ind_y_rmw::<ops::Dcp>());
    set(&mut t, 0xD7, &zp_x_rmw::<ops::Dcp>());
    set(&mut t, 0xDB, &abs_y_rmw::<ops::Dcp>());
    set(&mut t, 0xDF, &abs_x_rmw::<ops::Dcp>());

    // ISC — INC + SBC
    set(&mut t, 0xE3, &ind_x_rmw::<ops::Isc>());
    set(&mut t, 0xE7, &zp_rmw::<ops::Isc>());
    set(&mut t, 0xEF, &abs_rmw::<ops::Isc>());
    set(&mut t, 0xF3, &ind_y_rmw::<ops::Isc>());
    set(&mut t, 0xF7, &zp_x_rmw::<ops::Isc>());
    set(&mut t, 0xFB, &abs_y_rmw::<ops::Isc>());
    set(&mut t, 0xFF, &abs_x_rmw::<ops::Isc>());

    // LAX — LDA + LDX
    set(&mut t, 0xA3, &ind_x_read::<ops::Lax>());
    set(&mut t, 0xA7, &zp_read::<ops::Lax>());
    set(&mut t, 0xAF, &abs_read::<ops::Lax>());
    set(&mut t, 0xB3, &ind_y_read::<ops::Lax>());
    set(&mut t, 0xB7, &zp_y_read::<ops::Lax>());
    set(&mut t, 0xBF, &abs_y_read::<ops::Lax>());

    // SAX — store A & X
    set(&mut t, 0x83, &ind_x_write::<ops::Sax>());
    set(&mut t, 0x87, &zp_write::<ops::Sax>());
    set(&mut t, 0x8F, &abs_write::<ops::Sax>());
    set(&mut t, 0x97, &zp_y_write::<ops::Sax>());

    // Illegal immediate ops
    set(&mut t, 0x0B, &imm_read::<ops::Anc>());
    set(&mut t, 0x2B, &imm_read::<ops::Anc>());
    set(&mut t, 0x4B, &imm_read::<ops::Alr>());
    set(&mut t, 0x6B, &imm_read::<ops::Arr>());
    set(&mut t, 0x8B, &imm_read::<ops::Ane>());
    set(&mut t, 0xAB, &imm_read::<ops::Lxa>());
    set(&mut t, 0xCB, &imm_read::<ops::Axs>());
    set(&mut t, 0xEB, &imm_read::<ops::Usbc>());

    // LAS — A = X = SP = M & SP
    set(&mut t, 0xBB, &abs_y_read::<ops::Las>());

    // Unstable stores
    set(&mut t, 0x93, &ind_y_write::<ops::Sha>());
    set(&mut t, 0x9F, &abs_y_write::<ops::Sha>());
    set(&mut t, 0x9E, &abs_y_write::<ops::Shx>());
    set(&mut t, 0x9C, &abs_x_write::<ops::Shy>());
    set(&mut t, 0x9B, &tas_op());

    // Illegal NOPs — implied (2 cycles)
    set(&mut t, 0x1A, &imp::<ops::Nop>());
    set(&mut t, 0x3A, &imp::<ops::Nop>());
    set(&mut t, 0x5A, &imp::<ops::Nop>());
    set(&mut t, 0x7A, &imp::<ops::Nop>());
    set(&mut t, 0xDA, &imp::<ops::Nop>());
    set(&mut t, 0xFA, &imp::<ops::Nop>());

    // Illegal NOPs — immediate (2 cycles, read and discard)
    set(&mut t, 0x80, &imm_read::<ops::Nrd>());
    set(&mut t, 0x82, &imm_read::<ops::Nrd>());
    set(&mut t, 0x89, &imm_read::<ops::Nrd>());
    set(&mut t, 0xC2, &imm_read::<ops::Nrd>());
    set(&mut t, 0xE2, &imm_read::<ops::Nrd>());

    // Illegal NOPs — zero page (3 cycles, read and discard)
    set(&mut t, 0x04, &zp_read::<ops::Nrd>());
    set(&mut t, 0x44, &zp_read::<ops::Nrd>());
    set(&mut t, 0x64, &zp_read::<ops::Nrd>());

    // Illegal NOPs — zero page,X (4 cycles, read and discard)
    set(&mut t, 0x14, &zp_x_read::<ops::Nrd>());
    set(&mut t, 0x34, &zp_x_read::<ops::Nrd>());
    set(&mut t, 0x54, &zp_x_read::<ops::Nrd>());
    set(&mut t, 0x74, &zp_x_read::<ops::Nrd>());
    set(&mut t, 0xD4, &zp_x_read::<ops::Nrd>());
    set(&mut t, 0xF4, &zp_x_read::<ops::Nrd>());

    // Illegal NOPs — absolute (4 cycles, read and discard)
    set(&mut t, 0x0C, &abs_read::<ops::Nrd>());

    // Illegal NOPs — absolute,X (4+1 cycles, read and discard)
    set(&mut t, 0x1C, &abs_x_read::<ops::Nrd>());
    set(&mut t, 0x3C, &abs_x_read::<ops::Nrd>());
    set(&mut t, 0x5C, &abs_x_read::<ops::Nrd>());
    set(&mut t, 0x7C, &abs_x_read::<ops::Nrd>());
    set(&mut t, 0xDC, &abs_x_read::<ops::Nrd>());
    set(&mut t, 0xFC, &abs_x_read::<ops::Nrd>());

    // Dedicated fetch_opcode entry point in the last slot (opcode $FF step 7).
    // Used by set_pc to bootstrap into the normal phi1/phi2 loop.
    // (ISC abs,X only uses 7 steps, so step 7 is free.)
    t.steps[TABLE_SIZE - 1] = fetch_opcode;

    t
}
