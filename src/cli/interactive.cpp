#include "interactive.hpp"
#include "../core/encode.hpp"
#include <iostream>
#include <string>
#include <cstddef>

void run_interactive(const std::vector<Node>& tree,
                     const std::vector<WordCode>& guesses) {
    std::cout << "\n=== Interactive Mode ===" << std::endl;
    std::cout << "Enter feedback as 5 chars: g=green, y=yellow, b=gray" << std::endl;
    std::cout << "Example: bgybb" << std::endl;
    std::cout.flush();

    int node_idx = 0;
    while (true) {
        const Node& node = tree[static_cast<size_t>(node_idx)];
        std::string guess = decode_word(guesses[static_cast<size_t>(node.guess_idx)]);

        if (node.terminal) {
            std::cout << "\n  Guess: " << guess << "  (answer!)" << std::endl;
            break;
        }

        std::cout << "\n  Guess: " << guess << std::endl;
        std::cout << "  Feedback: ";
        std::cout.flush();

        std::string fb;
        std::cin >> fb;

        if (fb == "ggggg" || fb == "GGGGG") {
            std::cout << "  Solved!" << std::endl;
            break;
        }

        Pattern pat = 0;
        for (int i = 0; i < 5; i++) {
            size_t idx = static_cast<size_t>(i);
            int val = 0;
            if (fb[idx] == 'g' || fb[idx] == 'G') val = 2;
            else if (fb[idx] == 'y' || fb[idx] == 'Y') val = 1;
            pat = static_cast<Pattern>(pat + val * POW3[idx]);
        }

        int next = node.children[pat];
        if (next < 0) {
            std::cout << "  No path for that feedback \u2014 sure it\u2019s right?" << std::endl;
            break;
        }
        node_idx = next;
    }
}
