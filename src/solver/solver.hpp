#pragma once
#include "../core/types.hpp"
#include "../tree/node.hpp"
#include <vector>

int solve_one(const std::vector<Node>& tree,
              const std::vector<uint8_t>& pattern_cache,
              int secret, int num_answers);
