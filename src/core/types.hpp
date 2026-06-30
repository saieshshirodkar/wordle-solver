#pragma once
#include <cstdint>
#include <cstddef>

using Pattern = uint8_t;
using WordCode = uint32_t;

constexpr int WORD_LEN = 5;
constexpr int NUM_PATTERNS = 243;
constexpr int NUM_LETTERS = 26;
constexpr Pattern ALL_GREEN = 242;
constexpr Pattern POW3[WORD_LEN] = {1, 3, 9, 27, 81};
