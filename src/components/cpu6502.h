#ifndef COMPONENTS_CPU6502_H_
#define COMPONENTS_CPU6502_H_

#include "base/defines.h"


typedef struct cpu6502_t {
    uint16_t state;
    uint16_t ab;
    uint16_t pc;
    uint8_t a;
    uint8_t x;
    uint8_t y;
    uint8_t p;
    uint8_t s;
} cpu6502_t;


typedef struct cpu6502_in_t {
    uint8_t db;
    uint8_t irq : 1;
    uint8_t nmi : 1;
    uint8_t rst : 1;
} cpu6502_in_t;


typedef struct cpu6502_out_t {
    uint16_t ab;
    uint8_t db;
    uint8_t rw : 1;
    uint8_t sync : 1;
} cpu6502_out_t;


extern cpu6502_out_t (*cpu6502_fns[256][8])(cpu6502_t *, cpu6502_in_t);




#endif // ifndef COMPONENTS_CPU6502_H_
