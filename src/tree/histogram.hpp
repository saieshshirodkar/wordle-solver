#pragma once
#include "../core/types.hpp"
#include <cstdint>
#include <span>
#include <vector>

void build_histogram(int g, std::span<const int> remaining,
                     const std::vector<Pattern>& matrix,
                     int num_answers, uint16_t* hist);
