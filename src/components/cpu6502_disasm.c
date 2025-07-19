#include "components/cpu6502_disasm.h"


enum opcode {
    opcode_adc, opcode_and, opcode_asl, opcode_bcc,
    opcode_bcs, opcode_beq, opcode_bit, opcode_bmi,
    opcode_bne, opcode_bpl, opcode_brk, opcode_bvc,
    opcode_bvs, opcode_clc, opcode_cld, opcode_cli,
    opcode_clv, opcode_cmp, opcode_cpx, opcode_cpy,
    opcode_dec, opcode_dex, opcode_dey, opcode_eor,
    opcode_inc, opcode_inx, opcode_iny, opcode_jmp,
    opcode_jsr, opcode_lda, opcode_ldx, opcode_ldy,
    opcode_lsr, opcode_nop, opcode_ora, opcode_pha,
    opcode_php, opcode_pla, opcode_plp, opcode_rol,
    opcode_ror, opcode_rti, opcode_rts, opcode_sbc,
    opcode_sec, opcode_sed, opcode_sei, opcode_sta,
    opcode_stx, opcode_sty, opcode_tax, opcode_tay,
    opcode_tsx, opcode_txa, opcode_txs, opcode_tya,
    // illegal ops follow
    opcode_alr, opcode_anc, opcode_ane, opcode_arr,
    opcode_dcp, opcode_isc, opcode_kil, opcode_las,
    opcode_lax, opcode_lxa, opcode_rla, opcode_rra,
    opcode_sax, opcode_sbx, opcode_sha, opcode_shx,
    opcode_shy, opcode_slo, opcode_sre, opcode_tas
};

static const char opcode_name[][4] = {
    "ADC", "AND", "ASL", "BCC",
    "BCS", "BEQ", "BIT", "BMI",
    "BNE", "BPL", "BRK", "BVC",
    "BVS", "CLC", "CLD", "CLI",
    "CLV", "CMP", "CPX", "CPY",
    "DEC", "DEX", "DEY", "EOR",
    "INC", "INX", "INY", "JMP",
    "JSR", "LDA", "LDX", "LDY",
    "LSR", "NOP", "ORA", "PHA",
    "PHP", "PLA", "PLP", "ROL",
    "ROR", "RTI", "RTS", "SBC",
    "SEC", "SED", "SEI", "STA",
    "STX", "STY", "TAX", "TAY",
    "TSX", "TXA", "TXS", "TYA",
    // illegal ops
    "ALR", "ANC", "ANE", "ARR",
    "DCP", "ISC", "KIL", "LAS",
    "LAX", "LXA", "RLA", "RRA",
    "SAX", "SBX", "SHA", "SHX",
    "SHY", "SLO", "SRE", "TAS"
};

enum mode {
    mode_imp,  mode_acc,  mode_imm,  mode_zp,
    mode_zpx,  mode_zpy,  mode_abs,  mode_absx,
    mode_absy, mode_indx, mode_indy, mode_ind,
    mode_rel
};

static const uint8_t opcode_bytes[] = {
    1, 1, 2, 2,
    2, 2, 3, 3,
    3, 2, 2, 3,
    2
};

// Single byte, two cycle opcodes
#define imp(type)       {opcode_##type, mode_imp}
#define acc(type)       {opcode_##type, mode_acc}

// Load addressing modes
#define imm(type)       {opcode_##type, mode_imm}
#define lzp(type)       {opcode_##type, mode_zp}
#define lzpx(type)      {opcode_##type, mode_zpx}
#define lzpy(type)      {opcode_##type, mode_zpy}
#define labs(type)      {opcode_##type, mode_abs}
#define labsx(type)     {opcode_##type, mode_absx}
#define labsy(type)     {opcode_##type, mode_absy}
#define lindx(type)     {opcode_##type, mode_indx}
#define lindy(type)     {opcode_##type, mode_indy}

// Store addressing modes
#define szp(type)       {opcode_##type, mode_zp}
#define szpx(type)      {opcode_##type, mode_zpx}
#define szpy(type)      {opcode_##type, mode_zpy}
#define sabs(type)      {opcode_##type, mode_abs}
#define sabsx(type)     {opcode_##type, mode_absx}
#define sabsy(type)     {opcode_##type, mode_absy}
#define sindx(type)     {opcode_##type, mode_indx}
#define sindy(type)     {opcode_##type, mode_indy}

// Modify addressing mods
#define mzp(type)       {opcode_##type, mode_zp}
#define mzpx(type)      {opcode_##type, mode_zpx}
#define mabs(type)      {opcode_##type, mode_abs}
#define mabsx(type)     {opcode_##type, mode_absx}
#define mabsy(type)     {opcode_##type, mode_absy}
#define mindx(type)     {opcode_##type, mode_indx}
#define mindy(type)     {opcode_##type, mode_indy}

// Miscellaneous
#define push(type)      {opcode_##type, mode_imp}
#define pull(type)      {opcode_##type, mode_imp}
#define bra(type)       {opcode_##type, mode_rel}

// Special
#define brk()           {opcode_brk, mode_imp}
#define jsr()           {opcode_jsr, mode_abs}
#define jmp()           {opcode_jmp, mode_abs}
#define jmpind()        {opcode_jmp, mode_ind}
#define kil()           {opcode_kil, mode_imp}
#define rts()           {opcode_rts, mode_imp}
#define rti()           {opcode_rti, mode_imp}


cpu6502_opcode_t cpu6502_opcode_defs[256] = {
    #include "components/cpu6502_opcode_table.h"
};
