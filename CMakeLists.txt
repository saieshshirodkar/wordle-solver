cmake_minimum_required(VERSION 3.20)
project(wordle-solver LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

find_package(OpenMP)
set(CMAKE_INTERPROCEDURAL_OPTIMIZATION TRUE)

set(COMMON_SOURCES
    src/core/encode.cpp
    src/core/wordlist.cpp
    src/core/pattern.cpp
    src/core/matrix.cpp
    src/tree/histogram.cpp
    src/tree/scorer.cpp
    src/tree/tiebreaker.cpp
    src/tree/builder.cpp
    src/solver/solver.cpp
    src/solver/benchmark.cpp
)

add_executable(wordle-solver
    ${COMMON_SOURCES}
    src/cli/main.cpp
    src/cli/interactive.cpp
)

add_executable(wordle-test
    ${COMMON_SOURCES}
    src/test/test.cpp
)

target_include_directories(wordle-solver PRIVATE src)
target_include_directories(wordle-test PRIVATE src)

target_compile_options(wordle-solver PRIVATE -O3 -march=native -mtune=native -flto -Wall -Wextra -Wpedantic -Wconversion -Wsign-conversion -Wshadow -Wstrict-aliasing=2)
target_compile_options(wordle-test PRIVATE -O3 -march=native -mtune=native -flto -Wall -Wextra -Wpedantic -Wconversion -Wsign-conversion -Wshadow -Wstrict-aliasing=2)

if(OpenMP_CXX_FOUND)
    target_link_libraries(wordle-solver PRIVATE OpenMP::OpenMP_CXX)
    target_link_libraries(wordle-test PRIVATE OpenMP::OpenMP_CXX)
    target_compile_options(wordle-solver PRIVATE -fopenmp)
    target_compile_options(wordle-test PRIVATE -fopenmp)
endif()
