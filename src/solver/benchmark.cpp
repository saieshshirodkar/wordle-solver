#include "benchmark.hpp"
#include "solver.hpp"
#include <numeric>
#include <algorithm>
#include <cmath>

BenchmarkResult run_benchmark(
    const std::vector<Node>& tree,
    const std::vector<uint8_t>& pattern_cache,
    int num_answers
) {
    std::vector<int> guess_counts;
    guess_counts.reserve(static_cast<size_t>(num_answers));

    for (int secret = 0; secret < num_answers; secret++) {
        int gc = solve_one(tree, pattern_cache, secret, num_answers);
        guess_counts.push_back(gc);
    }

    double sum = static_cast<double>(std::accumulate(guess_counts.begin(), guess_counts.end(), 0ULL));
    double avg = sum / static_cast<double>(guess_counts.size());

    double sq_sum = 0.0;
    for (int gc : guess_counts)
        sq_sum += (gc - avg) * (gc - avg);
    double stddev = std::sqrt(sq_sum / static_cast<double>(guess_counts.size()));

    int worst = *std::max_element(guess_counts.begin(), guess_counts.end());

    return {avg, stddev, worst, std::move(guess_counts)};
}
