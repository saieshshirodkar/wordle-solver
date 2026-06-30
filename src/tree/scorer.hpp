#pragma once
#include <cstdint>

double score_histogram(const uint16_t* hist, const double* plogp);
void precompute_plogp(double* plogp, int max_count);
