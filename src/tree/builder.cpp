#include "builder.hpp"
#include "histogram.hpp"
#include "scorer.hpp"
#include "tiebreaker.hpp"
#include "../core/encode.hpp"
#include <numeric>
#include <algorithm>
#include <limits>
#include <cstring>
#include <cmath>

TreeBuilder::TreeBuilder(
    const std::vector<WordCode>& answers,
    const std::vector<WordCode>& guesses,
    const std::vector<Pattern>& matrix
)
    : answers_(answers)
    , guesses_(guesses)
    , matrix_(matrix)
    , num_answers_(static_cast<int>(answers_.size()))
    , num_guesses_(static_cast<int>(guesses_.size()))
    , num_allowed_(static_cast<int>(guesses_.size() - answers_.size()))
{
    precompute_plogp(plogp_, 2315);
    root_guess_idx_ = -1;
    WordCode soare = encode_word("soare");
    for (int i = 0; i < num_guesses_; i++) {
        if (guesses_[static_cast<size_t>(i)] == soare) {
            root_guess_idx_ = i;
            break;
        }
    }
}

std::vector<Node> TreeBuilder::build() {
    nodes_.clear();
    nodes_.reserve(5000);
    pattern_cache_.resize(static_cast<size_t>(5000) * static_cast<size_t>(num_answers_));

    std::vector<int> all_remaining(static_cast<size_t>(num_answers_));
    std::iota(all_remaining.begin(), all_remaining.end(), 0);

    build_node(all_remaining, 0);

    pattern_cache_.resize(nodes_.size() * static_cast<size_t>(num_answers_));
    return std::move(nodes_);
}

int TreeBuilder::build_node(std::span<const int> remaining, int depth) {
    int n_rem = static_cast<int>(remaining.size());

    if (n_rem == 1) {
        Node node;
        node.guess_idx = num_allowed_ + remaining[0];
        std::fill(std::begin(node.children), std::end(node.children), static_cast<int16_t>(-1));
        node.terminal = true;
        int idx = static_cast<int>(nodes_.size());
        nodes_.push_back(node);
        return idx;
    }

    if (n_rem == 2) {
        int guess_idx = num_allowed_ + remaining[0];
        Pattern other_pat = matrix_[static_cast<size_t>(guess_idx) * static_cast<size_t>(num_answers_) + static_cast<size_t>(remaining[1])];

        Node node;
        node.guess_idx = guess_idx;
        std::fill(std::begin(node.children), std::end(node.children), static_cast<int16_t>(-1));
        node.terminal = false;
        int my_idx = static_cast<int>(nodes_.size());
        nodes_.push_back(node);

        const Pattern* src = &matrix_[static_cast<size_t>(guess_idx) * static_cast<size_t>(num_answers_)];
        std::copy(src, src + num_answers_, &pattern_cache_[static_cast<size_t>(my_idx) * static_cast<size_t>(num_answers_)]);

        Node child;
        child.guess_idx = num_allowed_ + remaining[1];
        std::fill(std::begin(child.children), std::end(child.children), static_cast<int16_t>(-1));
        child.terminal = true;
        int cidx = static_cast<int>(nodes_.size());
        nodes_.push_back(child);
        nodes_[static_cast<size_t>(my_idx)].children[other_pat] = static_cast<int16_t>(cidx);

        return my_idx;
    }

    int best_guess;
    if (depth == 0 && root_guess_idx_ >= 0) {
        best_guess = root_guess_idx_;
    } else {
        std::vector<uint8_t> is_ans(static_cast<size_t>(num_guesses_), 0);
        for (int idx : remaining)
            is_ans[static_cast<size_t>(num_allowed_) + static_cast<size_t>(idx)] = 1;
        best_guess = find_best_guess(remaining, is_ans);
    }

    Node node;
    node.guess_idx = best_guess;
    std::fill(std::begin(node.children), std::end(node.children), static_cast<int16_t>(-1));
    node.terminal = false;
    int my_idx = static_cast<int>(nodes_.size());
    nodes_.push_back(node);

    const Pattern* src = &matrix_[static_cast<size_t>(best_guess) * static_cast<size_t>(num_answers_)];
    std::copy(src, src + num_answers_, &pattern_cache_[static_cast<size_t>(my_idx) * static_cast<size_t>(num_answers_)]);

    const Pattern* row = src;
    uint16_t counts[243] = {0};
    for (int idx : remaining)
        counts[row[static_cast<size_t>(idx)]]++;

    int total = 0;
    for (int p = 0; p < 243; p++) {
        if (p != ALL_GREEN)
            total += counts[p];
    }

    std::vector<int> buffer(static_cast<size_t>(total));
    int offsets[243];
    int pos = 0;
    for (int p = 0; p < 243; p++) {
        offsets[p] = pos;
        if (p != ALL_GREEN)
            pos += counts[p];
    }

    size_t write_pos[243];
    for (int p = 0; p < 243; p++)
        write_pos[p] = static_cast<size_t>(offsets[p]);
    for (int idx : remaining) {
        Pattern p = row[static_cast<size_t>(idx)];
        if (p != ALL_GREEN)
            buffer[write_pos[p]++] = idx;
    }

    for (int p = 0; p < 243; p++) {
        if (p == ALL_GREEN || counts[p] == 0)
            continue;
        int child_idx = build_node(
            std::span<const int>(buffer.data() + offsets[p], counts[p]),
            depth + 1
        );
        nodes_[static_cast<size_t>(my_idx)].children[p] = static_cast<int16_t>(child_idx);
    }

    return my_idx;
}

int TreeBuilder::find_best_guess(std::span<const int> remaining,
                                  const std::vector<uint8_t>& is_ans) {
    int n_rem = static_cast<int>(remaining.size());
    int best_guess = -1;
    double best_score = std::numeric_limits<double>::max();
    uint16_t best_hist[243];
    std::atomic<bool> found_perfect{false};

    auto worker = [&](int g, uint16_t* hist, bool thread_local_mode) -> bool {
        if (thread_local_mode && found_perfect.load(std::memory_order_relaxed))
            return false;

        build_histogram(g, remaining, matrix_, num_answers_, hist);
        double score = score_histogram(hist, plogp_);

        bool g_is_answer = is_ans[static_cast<size_t>(g)] != 0;

        if (score == 0.0 && g_is_answer) {
            found_perfect.store(true, std::memory_order_relaxed);
            best_guess = g;
            best_score = score;
            std::copy(hist, hist + 243, best_hist);
            return true;
        }

        bool better = false;
        if (best_guess == -1) {
            better = true;
        } else if (score < best_score - 1e-12) {
            better = true;
        } else if (std::abs(score - best_score) <= 1e-12) {
            better = tie_breaker(hist, best_hist, g_is_answer, is_ans[static_cast<size_t>(best_guess)] != 0);
        }

        if (better) {
            best_score = score;
            best_guess = g;
            std::copy(hist, hist + 243, best_hist);
        }

        return better;
    };

    if (n_rem <= 5) {
        uint16_t hist[243];
        for (int g = 0; g < num_guesses_; g++) {
            if (found_perfect.load(std::memory_order_relaxed))
                break;
            worker(g, hist, false);
        }
    } else {
        #pragma omp parallel
        {
            uint16_t hist[243];
            int local_best = -1;
            double local_best_score = std::numeric_limits<double>::max();
            uint16_t local_best_hist[243];

            #pragma omp for schedule(dynamic, 128) nowait
            for (int g = 0; g < num_guesses_; g++) {
                if (found_perfect.load(std::memory_order_relaxed))
                    continue;

                build_histogram(g, remaining, matrix_, num_answers_, hist);
                double score = score_histogram(hist, plogp_);

                bool g_is_answer = is_ans[static_cast<size_t>(g)] != 0;

                if (score == 0.0 && g_is_answer) {
                    #pragma omp critical
                    {
                        if (best_guess == -1 || g < best_guess) {
                            best_guess = g;
                            best_score = score;
                            std::copy(hist, hist + 243, best_hist);
                        }
                    }
                    continue;
                }

                bool better = false;
                if (local_best == -1) {
                    better = true;
                } else if (score < local_best_score - 1e-12) {
                    better = true;
                } else if (std::abs(score - local_best_score) <= 1e-12) {
                    better = tie_breaker(hist, local_best_hist, g_is_answer, is_ans[static_cast<size_t>(local_best)] != 0);
                }

                if (better) {
                    local_best_score = score;
                    local_best = g;
                    std::copy(hist, hist + 243, local_best_hist);
                }
            }

            #pragma omp critical
            {
                if (local_best != -1) {
                    if (best_guess == -1) {
                        best_score = local_best_score;
                        best_guess = local_best;
                        std::copy(local_best_hist, local_best_hist + 243, best_hist);
                    } else if (local_best_score < best_score - 1e-12) {
                        best_score = local_best_score;
                        best_guess = local_best;
                        std::copy(local_best_hist, local_best_hist + 243, best_hist);
                    } else if (std::abs(local_best_score - best_score) <= 1e-12) {
                        bool local_is_ans = is_ans[static_cast<size_t>(local_best)] != 0;
                        bool best_is_ans = is_ans[static_cast<size_t>(best_guess)] != 0;
                        if (tie_breaker(local_best_hist, best_hist, local_is_ans, best_is_ans)) {
                            best_score = local_best_score;
                            best_guess = local_best;
                            std::copy(local_best_hist, local_best_hist + 243, best_hist);
                        } else if (!tie_breaker(best_hist, local_best_hist, best_is_ans, local_is_ans) && local_best < best_guess) {
                            best_score = local_best_score;
                            best_guess = local_best;
                            std::copy(local_best_hist, local_best_hist + 243, best_hist);
                        }
                    }
                }
            }
        }
    }

    return best_guess;
}
