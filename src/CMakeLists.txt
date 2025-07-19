cmake_minimum_required(VERSION 3.31)
project("tawny")

set(CMAKE_C_STANDARD 17)

add_executable("tawny")

# Set compiler-specific properties
target_compile_options("tawny" PRIVATE
	"$<$<C_COMPILER_ID:MSVC>:/W4;/WX;/wd4100;/wd4820;/wd4189;/wd4206;/wd4101>"
	"$<$<C_COMPILER_ID:Clang,GNU>:-Wall;-Wextra;-Wpedantic;-Werror;-Wno-unused-but-set-variable;-Wno-unused-parameter;-Wno-unused-variable;-Wno-unused-function>"
)

# Set defines identifying the platform and the compiler being used
target_compile_definitions("tawny" PRIVATE
	"$<$<PLATFORM_ID:Windows>:PLATFORM_WINDOWS=1;PLATFORM_MACOS=0;PLATFORM_LINUX=0>"
	"$<$<PLATFORM_ID:Darwin>:PLATFORM_WINDOWS=0;PLATFORM_MACOS=1;PLATFORM_LINUX=0>"
	"$<$<PLATFORM_ID:Linux>:PLATFORM_WINDOWS=0;PLATFORM_MACOS=0;PLATFORM_LINUX=1>"
	"$<$<C_COMPILER_ID:MSVC>:COMPILER_MSVC=1;COMPILER_CLANG=0;COMPILER_GCC=0>"
	"$<$<C_COMPILER_ID:Clang>:COMPILER_MSVC=0;COMPILER_CLANG=1;COMPILER_GCC=0>"
	"$<$<C_COMPILER_ID:GNU>:COMPILER_MSVC=0;COMPILER_CLANG=0;COMPILER_GCC=1>"
)

# Determine whether tests are enabled for this build, and set a macro accordingly
set(TESTS_ENABLED ON CACHE BOOL "Build and run tests")
if(TESTS_ENABLED)
	target_compile_definitions("tawny" PRIVATE "TESTS_ENABLED=1")
endif()

target_include_directories("tawny" PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
target_link_libraries("tawny" PRIVATE "m")

add_subdirectory("app")
add_subdirectory("base")
add_subdirectory("components")
add_subdirectory("math")
add_subdirectory("templates")
target_sources("tawny" PRIVATE "main.c")

if(TESTS_ENABLED)
    add_subdirectory("test")
	add_custom_command(
		TARGET "tawny"
		COMMENT "Run tawny tests"
		POST_BUILD
		COMMAND tawny --test
	)
endif()
