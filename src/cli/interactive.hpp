#pragma once
#include "../core/types.hpp"
#include "../tree/node.hpp"
#include <vector>

void run_interactive(const std::vector<Node>& tree,
                     const std::vector<WordCode>& guesses);
