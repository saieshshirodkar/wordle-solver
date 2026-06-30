#include "matrix.hpp"
#include "pattern.hpp"

std::vector<Pattern> build_pattern_matrix(
    const std::vector<WordCode>& guesses,
    const std::vector<WordCode>& answers
) {
    size_t ng = guesses.size();
    size_t na = answers.size();
    std::vector<Pattern> matrix(ng * na);

    #pragma omp parallel for schedule(dynamic, 32)
    for (size_t g = 0; g < ng; g++) {
        WordCode guess = guesses[g];
        size_t base = g * na;
        for (size_t a = 0; a < na; a++)
            matrix[base + a] = compute_pattern(guess, answers[a]);
    }

    return matrix;
}
