#include "../core/wordlist.hpp"
#include "../core/matrix.hpp"
#include "../core/encode.hpp"
#include "../tree/builder.hpp"
#include "../solver/benchmark.hpp"
#include "interactive.hpp"
#include <iostream>
#include <chrono>
#include <iomanip>
#include <cstddef>
#include <string>

int main(int argc, char* argv[]) {
    std::ios::sync_with_stdio(false);

    std::cout << "=== Wordle Solver ===\n" << std::endl;

    auto answers = load_words("data/answers.txt");
    auto allowed = load_words("data/allowed.txt");

    if (answers.empty() || allowed.empty()) {
        std::cerr << "Failed to load word lists from data/ directory.\n";
        return 1;
    }

    auto guesses = allowed;
    guesses.insert(guesses.end(), answers.begin(), answers.end());

    std::cout << "Answers: " << answers.size() << std::endl;
    std::cout << "Allowed guesses: " << allowed.size() << std::endl;
    std::cout << "Total guesses: " << guesses.size() << std::endl;
    std::cout.flush();

    auto t0 = std::chrono::high_resolution_clock::now();
    auto matrix = build_pattern_matrix(guesses, answers);
    auto t1 = std::chrono::high_resolution_clock::now();

    double matrix_ms = std::chrono::duration<double, std::milli>(t1 - t0).count();
    double matrix_mb = static_cast<double>(matrix.size() * sizeof(Pattern)) / (1024.0 * 1024.0);
    std::cout << "Pattern matrix: " << matrix_mb << " MB (" << matrix_ms << " ms)" << std::endl;
    std::cout.flush();

    t0 = std::chrono::high_resolution_clock::now();
    TreeBuilder builder(answers, guesses, matrix);
    auto tree = builder.build();
    const auto& pattern_cache = builder.pattern_cache();
    t1 = std::chrono::high_resolution_clock::now();

    double tree_ms = std::chrono::duration<double, std::milli>(t1 - t0).count();
    std::cout << "Tree nodes: " << tree.size() << " (" << tree_ms << " ms)" << std::endl;
    std::cout << "First guess: " << decode_word(guesses[static_cast<size_t>(tree[0].guess_idx)]) << std::endl;
    std::cout.flush();

    auto start = std::chrono::high_resolution_clock::now();
    auto result = run_benchmark(tree, pattern_cache, static_cast<int>(answers.size()));
    auto end = std::chrono::high_resolution_clock::now();

    double time_ms = std::chrono::duration<double, std::milli>(end - start).count();

    std::vector<int> hist(10, 0);
    for (int gc : result.guess_counts)
        if (gc < static_cast<int>(hist.size())) hist[static_cast<size_t>(gc)]++;

    std::cout << "\nGuess distribution:";
    for (int i = 1; i < static_cast<int>(hist.size()); i++)
        if (hist[static_cast<size_t>(i)] > 0)
            std::cout << "  " << i << ": " << hist[static_cast<size_t>(i)];
    std::cout << std::endl;

    std::cout << "\n=== Results ===" << std::endl;
    std::cout << "  Average:  " << std::fixed << std::setprecision(4) << result.avg_guesses << std::endl;
    std::cout << "  Stddev:   " << result.stddev_guesses << std::endl;
    std::cout << "  Worst:    " << result.worst_guesses << std::endl;
    std::cout << "  Total:    " << time_ms << " ms" << std::endl;
    std::cout << "  Per game: " << std::setprecision(3) << (time_ms / static_cast<double>(answers.size())) << " ms" << std::endl;

    size_t cache_kb = (pattern_cache.size() * sizeof(uint8_t)) / 1024;
    std::cout << "  Pattern cache: " << cache_kb << " KB" << std::endl;

    if (argc > 1) {
        std::string fb = argv[1];
        if (fb.size() != 5) {
            std::cerr << "Need 5-char feedback (g/y/b)\n";
            return 1;
        }
        Pattern pat = 0;
        for (int i = 0; i < 5; i++) {
            size_t idx = static_cast<size_t>(i);
            int val = 0;
            if (fb[idx] == 'g' || fb[idx] == 'G') val = 2;
            else if (fb[idx] == 'y' || fb[idx] == 'Y') val = 1;
            pat = static_cast<Pattern>(pat + val * POW3[idx]);
        }
        int next = tree[0].children[pat];
        if (next < 0) {
            std::cout << "No path for that feedback\n";
            return 1;
        }
        std::cout << decode_word(guesses[static_cast<size_t>(tree[static_cast<size_t>(next)].guess_idx)]) << std::endl;
        return 0;
    }

    run_interactive(tree, guesses);

    return 0;
}
