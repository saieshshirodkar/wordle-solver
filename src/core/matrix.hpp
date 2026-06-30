#pragma once
#include "types.hpp"
#include <vector>
#include <cstddef>

std::vector<Pattern> build_pattern_matrix(
    const std::vector<WordCode>& guesses,
    const std::vector<WordCode>& answers
);
