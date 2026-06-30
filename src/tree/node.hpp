#pragma once
#include "../core/types.hpp"
#include <cstdint>

struct Node {
    int guess_idx;
    int16_t children[243];
    bool terminal;
};
