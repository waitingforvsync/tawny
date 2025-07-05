#ifndef DEFINES_H_
#define DEFINES_H_

#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#if COMPILER_MSVC
#include <intrin.h>
#endif

#if COMPILER_MSVC
#define trap() __debugbreak()
#define unreachable() __assume(0)
#elif COMPILER_CLANG || COMPILER_GCC
#define trap() __builtin_trap()
#define unreachable() __builtin_unreachable()
#else
#error "Platform not supported"
#endif

#define check(cond) ((void)((cond) || (trap(), 0)))
#define require(cond) ((void)((cond) || (abort(), 0)))
#define static_check(cond, msg) _Static_assert(cond, msg)

#define ssizeof(x) ((int32_t)sizeof(x))

#define CONCAT(a, b) CONCAT2_(a, b)
#define CONCAT2_(a, b) a##b

#define STRINGIFY(a) STRINGIFY2_(a)
#define STRINGIFY2_(a) #a

#define INDEX_NONE (-1)


#endif // ifndef DEFINES_H_
