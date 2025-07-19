#include "components/cpu6502.h"


enum flag_bits {
    c_bit, z_bit, i_bit, d_bit, b_bit, u_bit, v_bit, n_bit
};

enum flags {
    c_flag = (1 << c_bit),
    z_flag = (1 << z_bit),
    i_flag = (1 << i_bit),
    d_flag = (1 << d_bit),
    b_flag = (1 << b_bit),
    u_flag = (1 << u_bit),
    v_flag = (1 << v_bit),
    n_flag = (1 << n_bit),
    // Combined flags
    nz_flags = (n_flag | z_flag),
    nv_flags = (n_flag | v_flag),
    nvz_flags = (n_flag | v_flag | z_flag),
    nvzc_flags = (n_flag | v_flag | z_flag | c_flag)
};


static cpu6502_out_t get_opcode(cpu6502_t *cpu, cpu6502_in_t in) {
    cpu->state = in.db * 8;
    return (cpu6502_out_t) {.ab = ++cpu->pc, .sync = 1};
}

static cpu6502_out_t get_ea_lzp(cpu6502_t *cpu, cpu6502_in_t in) {
    cpu->pc++;
    return (cpu6502_out_t) {.ab = cpu->ab = in.db};
}

static cpu6502_out_t get_ea_labs_l(cpu6502_t *cpu, cpu6502_in_t in) {
    cpu->pc++;
    cpu->ab = in.db;
    return (cpu6502_out_t) {.ab = cpu->pc};
}

static cpu6502_out_t get_ea_labs_h(cpu6502_t *cpu, cpu6502_in_t in) {
    cpu->pc++;
    return (cpu6502_out_t) {.ab = cpu->ab | (in.db << 8)};
}

static cpu6502_out_t get_ea_labsx_h(cpu6502_t *cpu, cpu6502_in_t in) {
    cpu->pc++;
    cpu->ab = in.db;
    return (cpu6502_out_t) {.ab = cpu->pc};
}

static cpu6502_out_t get_ea_labsx_pc(cpu6502_t *cpu, cpu6502_in_t in) {
    cpu->pc++;
    cpu->ab = in.db;
    return (cpu6502_out_t) {.ab = cpu->pc};
}

static inline void set_nz(cpu6502_t *cpu, uint8_t val) {
    cpu->p = (cpu->p & ~nz_flags) | (val & n_flag) | ((val == 0) << z_bit);
}

static cpu6502_out_t op_clc(cpu6502_t *cpu, cpu6502_in_t in) { cpu->p &= ~c_flag; return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_cld(cpu6502_t *cpu, cpu6502_in_t in) { cpu->p &= ~d_flag; return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_cli(cpu6502_t *cpu, cpu6502_in_t in) { cpu->p &= ~i_flag; return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_clv(cpu6502_t *cpu, cpu6502_in_t in) { cpu->p &= ~v_flag; return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_dex(cpu6502_t *cpu, cpu6502_in_t in) { set_nz(cpu, --cpu->x); return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_dey(cpu6502_t *cpu, cpu6502_in_t in) { set_nz(cpu, --cpu->y); return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_inx(cpu6502_t *cpu, cpu6502_in_t in) { set_nz(cpu, ++cpu->x); return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_iny(cpu6502_t *cpu, cpu6502_in_t in) { set_nz(cpu, ++cpu->y); return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_sec(cpu6502_t *cpu, cpu6502_in_t in) { cpu->p |= c_flag; return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_sed(cpu6502_t *cpu, cpu6502_in_t in) { cpu->p |= d_flag; return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_sei(cpu6502_t *cpu, cpu6502_in_t in) { cpu->p |= i_flag; return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_tax(cpu6502_t *cpu, cpu6502_in_t in) { set_nz(cpu, cpu->x = cpu->a); return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_tay(cpu6502_t *cpu, cpu6502_in_t in) { set_nz(cpu, cpu->y = cpu->a); return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_txa(cpu6502_t *cpu, cpu6502_in_t in) { set_nz(cpu, cpu->a = cpu->x); return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_tya(cpu6502_t *cpu, cpu6502_in_t in) { set_nz(cpu, cpu->a = cpu->y); return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_txs(cpu6502_t *cpu, cpu6502_in_t in) { cpu->s = cpu->x; return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_tsx(cpu6502_t *cpu, cpu6502_in_t in) { set_nz(cpu, cpu->x = cpu->s); return (cpu6502_out_t) {.ab = cpu->pc}; }

static cpu6502_out_t op_and(cpu6502_t *cpu, cpu6502_in_t in) { set_nz(cpu, cpu->a &= in.db); return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_bit(cpu6502_t *cpu, cpu6502_in_t in) { cpu->p = (cpu->p & ~nvz_flags) | (in.db & nv_flags) | (!(cpu->a & in.db) << z_bit); return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_eor(cpu6502_t *cpu, cpu6502_in_t in) { set_nz(cpu, cpu->a ^= in.db); return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_lax(cpu6502_t *cpu, cpu6502_in_t in) { set_nz(cpu, cpu->a = cpu->x = in.db); return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_lda(cpu6502_t *cpu, cpu6502_in_t in) { set_nz(cpu, cpu->a = in.db); return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_ldx(cpu6502_t *cpu, cpu6502_in_t in) { set_nz(cpu, cpu->x = in.db); return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_ldy(cpu6502_t *cpu, cpu6502_in_t in) { set_nz(cpu, cpu->y = in.db); return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_ora(cpu6502_t *cpu, cpu6502_in_t in) { set_nz(cpu, cpu->a |= in.db); return (cpu6502_out_t) {.ab = cpu->pc}; }
static cpu6502_out_t op_nop(cpu6502_t *cpu, cpu6502_in_t in) { return (cpu6502_out_t) {.ab = cpu->pc}; }

static cpu6502_out_t imm_and(cpu6502_t *cpu, cpu6502_in_t in) { cpu->pc++; return op_and(cpu, in); }
static cpu6502_out_t imm_eor(cpu6502_t *cpu, cpu6502_in_t in) { cpu->pc++; return op_eor(cpu, in); }
static cpu6502_out_t imm_lax(cpu6502_t *cpu, cpu6502_in_t in) { cpu->pc++; return op_lax(cpu, in); }
static cpu6502_out_t imm_lda(cpu6502_t *cpu, cpu6502_in_t in) { cpu->pc++; return op_lda(cpu, in); }
static cpu6502_out_t imm_ldx(cpu6502_t *cpu, cpu6502_in_t in) { cpu->pc++; return op_ldx(cpu, in); }
static cpu6502_out_t imm_ldy(cpu6502_t *cpu, cpu6502_in_t in) { cpu->pc++; return op_ldy(cpu, in); }
static cpu6502_out_t imm_ora(cpu6502_t *cpu, cpu6502_in_t in) { cpu->pc++; return op_ora(cpu, in); }
static cpu6502_out_t imm_nop(cpu6502_t *cpu, cpu6502_in_t in) { cpu->pc++; return op_nop(cpu, in); }


// Single byte, two cycle opcodes
#define imp(type)       {op_##type, get_opcode}
#define acc(type)       {0}

// Load addressing modes
#define imm(type)       {imm_##type, get_opcode}
#define lzp(type)       {get_ea_lzp, op_##type, get_opcode}
#define lzpx(type)      {0}
#define lzpy(type)      {0}
#define labs(type)      {get_ea_labs_l, get_ea_labs_h, op_##type, get_opcode}
#define labsx(type)     {get_ea_labs_l, get_ea_labsx_h, get_ea_labsx_pc, op_##type, get_opcode}
#define labsy(type)     {0}
#define lindx(type)     {0}
#define lindy(type)     {0}

// Store addressing modes
#define szp(type)       {0}
#define szpx(type)      {0}
#define szpy(type)      {0}
#define sabs(type)      {0}
#define sabsx(type)     {0}
#define sabsy(type)     {0}
#define sindx(type)     {0}
#define sindy(type)     {0}

// Modify addressing mods
#define mzp(type)       {0}
#define mzpx(type)      {0}
#define mabs(type)      {0}
#define mabsx(type)     {0}
#define mabsy(type)     {0}
#define mindx(type)     {0}
#define mindy(type)     {0}

// Miscellaneous
#define push(type)      {0}
#define pull(type)      {0}
#define bra(type)       {0}

// Special
#define brk()           {0}
#define jsr()           {0}
#define jmp()           {0}
#define jmpind()        {0}
#define kil()           {0}
#define rts()           {0}
#define rti()           {0}


cpu6502_out_t (*cpu6502_fns[256][8])(cpu6502_t *, cpu6502_in_t) = {
    #include "components/cpu6502_opcode_table.h"
};
