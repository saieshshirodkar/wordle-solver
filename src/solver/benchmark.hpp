#pragma once
#include "../core/types.hpp"
#include "../tree/node.hpp"
#include <vector>

struct BenchmarkResult {
    double avg_guesses;
    double stddev_guesses;
    int worst_guesses;
    std::vector<int> guess_counts;
};

BenchmarkResult run_benchmark(
    const std::vector<Node>& tree,
    const std::vector<uint8_t>& pattern_cache,
    int num_answers
);
