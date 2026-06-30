#include "scorer.hpp"
#include <cmath>

double score_histogram(const uint16_t* hist, const double* plogp) {
    double score = 0.0;
    for (int p = 0; p < 243; p++)
        score += plogp[hist[p]];
    return score;
}

void precompute_plogp(double* plogp, int max_count) {
    plogp[0] = 0.0;
    for (int c = 1; c <= max_count; c++)
        plogp[c] = static_cast<double>(c) * std::log2(static_cast<double>(c));
}
