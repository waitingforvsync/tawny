#ifndef COMPONENTS_CPU6502_DISASM_H_
#define COMPONENTS_CPU6502_DISASM_H_

#include "base/defines.h"


typedef struct cpu6502_opcode_t {
    uint8_t opcode;
    uint8_t mode;
} cpu6502_opcode_t;


extern cpu6502_opcode_t cpu6502_opcode_defs[256];



#endif // ifndef COMPONENTS_CPU6502_DISASM_H_
