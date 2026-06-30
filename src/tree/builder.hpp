#pragma once
#include "../core/types.hpp"
#include "node.hpp"
#include <vector>
#include <span>
#include <atomic>

class TreeBuilder {
public:
    TreeBuilder(
        const std::vector<WordCode>& answers,
        const std::vector<WordCode>& guesses,
        const std::vector<Pattern>& matrix
    );

    std::vector<Node> build();
    const std::vector<uint8_t>& pattern_cache() const { return pattern_cache_; }

private:
    int build_node(std::span<const int> remaining, int depth);
    int find_best_guess(std::span<const int> remaining, const std::vector<uint8_t>& is_ans);

    const std::vector<WordCode>& answers_;
    const std::vector<WordCode>& guesses_;
    const std::vector<Pattern>& matrix_;
    std::vector<Node> nodes_;
    std::vector<uint8_t> pattern_cache_;
    int num_answers_;
    int num_guesses_;
    int num_allowed_;
    double plogp_[2316];
    int root_guess_idx_;
};
