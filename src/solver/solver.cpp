#include "solver.hpp"

int solve_one(const std::vector<Node>& tree,
              const std::vector<uint8_t>& pattern_cache,
              int secret, int num_answers) {
    int node_idx = 0;
    int guesses = 0;

    while (true) {
        const Node& node = tree[static_cast<size_t>(node_idx)];
        guesses++;

        if (node.terminal)
            break;

        Pattern feedback = pattern_cache[static_cast<size_t>(node_idx) * static_cast<size_t>(num_answers) + static_cast<size_t>(secret)];

        if (feedback == ALL_GREEN)
            break;

        node_idx = node.children[feedback];
    }

    return guesses;
}
