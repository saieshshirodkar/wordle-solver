#include "histogram.hpp"
#include <cstring>

void build_histogram(int g, std::span<const int> remaining,
                     const std::vector<Pattern>& matrix,
                     int num_answers, uint16_t* hist) {
    std::fill(hist, hist + 243, 0);
    const Pattern* row = &matrix[static_cast<size_t>(g) * static_cast<size_t>(num_answers)];
    for (int idx : remaining)
        hist[row[idx]]++;
}
