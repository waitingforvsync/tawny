if(TESTS_ENABLED)
    add_subdirectory("test")
endif()

target_sources("tawny" PRIVATE
    "cpu6502.c"
    "cpu6502.h"
    "cpu6502_disasm.c"
    "cpu6502_disasm.h"
    "cpu6502_opcode_table.h"
)
