#include "../core/wordlist.hpp"
#include "../core/pattern.hpp"
#include "../core/matrix.hpp"
#include "../core/encode.hpp"
#include "../tree/builder.hpp"
#include "../tree/histogram.hpp"
#include "../tree/scorer.hpp"
#include "../tree/tiebreaker.hpp"
#include "../solver/solver.hpp"
#include "../solver/benchmark.hpp"
#include <iostream>
#include <string>
#include <cstring>
#include <cmath>
#include <algorithm>
#include <numeric>
#include <cstddef>

static int passed = 0, failed = 0;

#define TEST(name) do { std::cout << "  " << name << "... "; } while(0)
#define PASS() do { std::cout << "PASS\n"; passed++; } while(0)
#define FAIL(msg) do { std::cout << "FAIL: " << msg << "\n"; failed++; } while(0)
#define CHECK(cond, msg) do { if (!(cond)) { FAIL(msg); return; } } while(0)
#define CHECK_EQ(a, b, msg) do { if ((a) != (b)) { FAIL(msg << " (" << (a) << " != " << (b) << ")"); return; } } while(0)
#define CHECK_NEAR(a, b, eps, msg) do { if (std::abs((a) - (b)) > (eps)) { FAIL(msg << " (" << (a) << " - " << (b) << " > " << (eps) << ")"); return; } } while(0)

void test_encode_decode() {
    TEST("encode/decode roundtrip");
    std::string words[] = {"soare", "emcee", "crypt", "molar", "aahed", "abaca", "abbey"};
    for (auto& w : words) {
        CHECK_EQ(decode_word(encode_word(w)), w, "roundtrip failed for " << w);
    }
    PASS();
}

void test_compute_pattern() {
    TEST("all green");
    CHECK_EQ(compute_pattern(encode_word("abcde"), encode_word("abcde")), ALL_GREEN, "all green");
    PASS();

    TEST("all gray");
    CHECK_EQ(compute_pattern(encode_word("abcde"), encode_word("fghij")), 0, "all gray");
    PASS();

    TEST("one yellow");
    CHECK_EQ(compute_pattern(encode_word("abcde"), encode_word("fghci")), POW3[2], "only C yellow");
    PASS();

    TEST("duplicate letters - ABBEY vs ABOUT");
    CHECK_EQ(compute_pattern(encode_word("abbey"), encode_word("about")), static_cast<Pattern>(8), "ABBEY vs ABOUT");
    PASS();

    TEST("duplicate letters - ABBEY vs ABODE");
    CHECK_EQ(compute_pattern(encode_word("abbey"), encode_word("abode")), static_cast<Pattern>(35), "ABBEY vs ABODE");
    PASS();
}

void test_precompute_plogp() {
    double plogp[2316];
    precompute_plogp(plogp, 2315);

    TEST("plogp[0] = 0");
    CHECK_EQ(plogp[0], 0.0, "plogp[0] should be 0");
    PASS();

    TEST("plogp[1] = 0");
    CHECK_EQ(plogp[1], 0.0, "plogp[1] should be 0");
    PASS();

    TEST("plogp[2] = 2.0");
    CHECK_NEAR(plogp[2], 2.0, 1e-12, "plogp[2] should be 2.0");
    PASS();

    TEST("plogp[4] = 8.0");
    CHECK_NEAR(plogp[4], 8.0, 1e-12, "plogp[4] should be 8.0");
    PASS();
}

void test_score_histogram() {
    double plogp[2316];
    precompute_plogp(plogp, 2315);
    uint16_t hist[243];

    TEST("empty histogram = 0");
    std::fill(hist, hist + 243, 0);
    CHECK_EQ(score_histogram(hist, plogp), 0.0, "empty score should be 0");
    PASS();

    TEST("single bucket with 4 = 8.0");
    std::fill(hist, hist + 243, 0);
    hist[0] = 4;
    CHECK_NEAR(score_histogram(hist, plogp), 8.0, 1e-12, "score should be plogp[4]");
    PASS();

    TEST("two buckets: 4 + 2 = plogp[4] + plogp[2]");
    std::fill(hist, hist + 243, 0);
    hist[0] = 4;
    hist[1] = 2;
    CHECK_NEAR(score_histogram(hist, plogp), 8.0 + 2.0, 1e-12, "score should sum plogp");
    PASS();
}

void test_tie_breaker() {
    uint16_t h1[243] = {0};
    uint16_t h2[243] = {0};

    TEST("prefer answer over non-answer");
    CHECK(tie_breaker(h1, h2, true, false), "answer should win");
    CHECK(!tie_breaker(h1, h2, false, true), "non-answer should lose");
    PASS();

    TEST("lower max wins");
    h1[0] = 5; h2[0] = 3;
    CHECK(!tie_breaker(h1, h2, false, false), "new with higher max should lose");
    CHECK(tie_breaker(h2, h1, false, false), "new with lower max should win");
    PASS();

    TEST("more nonzero buckets wins when max tied");
    std::fill(h1, h1 + 243, 0);
    std::fill(h2, h2 + 243, 0);
    h1[0] = 5; h1[1] = 3;
    h2[0] = 5; h2[1] = 1; h2[2] = 1;
    CHECK(tie_breaker(h2, h1, false, false), "more nonzeros should win");
    CHECK(!tie_breaker(h1, h2, false, false), "fewer nonzeros should lose");
    PASS();

    TEST("identical hists: both false (lower index wins externally)");
    std::fill(h1, h1 + 243, 0);
    std::fill(h2, h2 + 243, 0);
    h1[5] = 10; h2[5] = 10;
    CHECK(!tie_breaker(h1, h2, false, false), "identical should be false");
    CHECK(!tie_breaker(h2, h1, false, false), "identical should be false");
    PASS();
}

void test_build_histogram(const std::vector<Pattern>& matrix,
                          int num_answers,
                          const std::vector<WordCode>& guesses) {
    {
        TEST("histogram sums to remaining count");
        std::vector<int> remaining(static_cast<size_t>(num_answers));
        std::iota(remaining.begin(), remaining.end(), 0);

        uint16_t hist[243];
        build_histogram(0, remaining, matrix, num_answers, hist);

        int sum = 0;
        for (int p = 0; p < 243; p++)
            sum += hist[p];
        CHECK_EQ(sum, num_answers, "histogram sum should equal remaining count");
        PASS();
    }

    {
        TEST("an answer-guess has ALL_GREEN for itself");
        int num_allowed = static_cast<int>(guesses.size()) - num_answers;
        int first_answer_idx = num_allowed;

        std::vector<int> remaining(static_cast<size_t>(num_answers));
        std::iota(remaining.begin(), remaining.end(), 0);

        uint16_t hist[243];
        build_histogram(first_answer_idx, remaining, matrix, num_answers, hist);
        CHECK(hist[ALL_GREEN] >= 1, "an answer-guess should match itself via ALL_GREEN");
        PASS();
    }
}

void test_matrix_dimensions(const std::vector<Pattern>& matrix,
                            const std::vector<WordCode>& guesses,
                            const std::vector<WordCode>& answers) {
    TEST("matrix dimensions");
    CHECK_EQ(matrix.size(), guesses.size() * answers.size(), "matrix size mismatch");
    PASS();

    TEST("matrix entry matches compute_pattern");
    Pattern p = matrix[0 * answers.size() + 0];
    Pattern expected = compute_pattern(guesses[0], answers[0]);
    CHECK_EQ(p, expected, "matrix[0][0] should match compute_pattern");
    PASS();
}

void test_solve_one_direct(const std::vector<Node>& tree,
                            const std::vector<uint8_t>& pattern_cache,
                            const std::vector<WordCode>& answers) {
    auto find_secret = [&](const std::string& word) -> int {
        WordCode code = encode_word(word);
        for (int i = 0; i < static_cast<int>(answers.size()); i++)
            if (answers[static_cast<size_t>(i)] == code) return i;
        return -1;
    };

    int na = static_cast<int>(answers.size());

    TEST("solve_one: emcee -> 4");
    CHECK_EQ(solve_one(tree, pattern_cache, find_secret("emcee"), na), 4, "emcee");
    PASS();

    TEST("solve_one: molar -> 3");
    CHECK_EQ(solve_one(tree, pattern_cache, find_secret("molar"), na), 3, "molar");
    PASS();

    TEST("solve_one: crypt -> 3");
    CHECK_EQ(solve_one(tree, pattern_cache, find_secret("crypt"), na), 3, "crypt");
    PASS();


}

void test_known_trace(const std::vector<WordCode>& answers,
                      const std::vector<Node>& tree,
                      const std::vector<uint8_t>& pattern_cache,
                      const std::string& word, int expected_guesses) {
    WordCode code = encode_word(word);
    int secret = -1;
    for (int i = 0; i < static_cast<int>(answers.size()); i++) {
        if (answers[static_cast<size_t>(i)] == code) { secret = i; break; }
    }

    int guess_count = 0;
    int node_idx = 0;
    while (true) {
        const Node& node = tree[static_cast<size_t>(node_idx)];
        guess_count++;
        if (node.terminal) break;
        Pattern feedback = pattern_cache[static_cast<size_t>(node_idx) * answers.size() + static_cast<size_t>(secret)];
        if (feedback == ALL_GREEN) break;
        node_idx = node.children[feedback];
    }

    CHECK_EQ(guess_count, expected_guesses,
             word << " expected " << expected_guesses << " guesses, got " << guess_count);
}

void test_traces(const std::vector<WordCode>& answers,
                 const std::vector<Node>& tree,
                 const std::vector<uint8_t>& pattern_cache) {
    TEST("known trace: emcee -> 4");
    test_known_trace(answers, tree, pattern_cache, "emcee", 4);
    PASS();

    TEST("known trace: molar -> 3");
    test_known_trace(answers, tree, pattern_cache, "molar", 3);
    PASS();

    TEST("known trace: crypt -> 3");
    test_known_trace(answers, tree, pattern_cache, "crypt", 3);
    PASS();

    TEST("known trace: which -> 4");
    test_known_trace(answers, tree, pattern_cache, "which", 4);
    PASS();
}

void test_all_words_solved(const std::vector<Node>& tree,
                           const std::vector<uint8_t>& pattern_cache,
                           int num_answers) {
    TEST("all " << num_answers << " words solvable");
    int max_depth = 0;
    for (int secret = 0; secret < num_answers; secret++) {
        int node_idx = 0;
        int guess_count = 0;
        while (true) {
            const Node& node = tree[static_cast<size_t>(node_idx)];
            guess_count++;
            CHECK(guess_count <= 10, "infinite loop at secret " << secret);
            if (node.terminal) break;
            Pattern feedback = pattern_cache[static_cast<size_t>(node_idx) * static_cast<size_t>(num_answers) + static_cast<size_t>(secret)];
            if (feedback == ALL_GREEN) break;
            node_idx = node.children[feedback];
        }
        if (guess_count > max_depth) max_depth = guess_count;
    }
    CHECK(max_depth <= 6, "worst case " << max_depth << " > 6");
    PASS();
}

void test_first_guess_is_soare(const std::vector<WordCode>& guesses,
                                const std::vector<Node>& tree) {
    TEST("first guess is 'soare'");
    CHECK_EQ(decode_word(guesses[static_cast<size_t>(tree[0].guess_idx)]), "soare",
             "first guess not soare");
    PASS();
}

void test_matrix_and_cache_coherence(const std::vector<Pattern>& matrix,
                                      const std::vector<uint8_t>& pattern_cache,
                                      int num_answers,
                                      const std::vector<Node>& tree) {
    TEST("pattern cache matches matrix for internal nodes");
    for (size_t ni = 0; ni < tree.size(); ni++) {
        const Node& node = tree[ni];
        if (node.terminal) continue;
        int g = node.guess_idx;
        for (int a = 0; a < num_answers; a++) {
            Pattern from_cache = pattern_cache[ni * static_cast<size_t>(num_answers) + static_cast<size_t>(a)];
            Pattern from_matrix = matrix[static_cast<size_t>(g) * static_cast<size_t>(num_answers) + static_cast<size_t>(a)];
            if (from_cache != from_matrix) {
                FAIL("cache mismatch at node " << ni << " answer " << a
                     << " (cache=" << (int)from_cache << " matrix=" << (int)from_matrix << ")");
                return;
            }
        }
    }
    PASS();
}

void test_no_invalid_child_indices(const std::vector<Node>& tree) {
    TEST("no invalid child indices");
    for (size_t ni = 0; ni < tree.size(); ni++) {
        const Node& node = tree[ni];
        for (int p = 0; p < 243; p++) {
            int16_t c = node.children[p];
            if (p == ALL_GREEN) {
                CHECK(c == -1, "children[242] should be -1 at node " << ni);
            } else if (c >= 0) {
                CHECK(static_cast<size_t>(c) < tree.size(),
                      "child index " << c << " out of bounds at node " << ni);
            }
        }
    }
    PASS();
}

void test_i32_repr_bits() {
    TEST("letter encoding uses 5 bits per position");
    WordCode code = encode_word("abcde");
    for (int i = 0; i < 5; i++)
        CHECK_EQ((code >> (i * 5)) & 0x1F, static_cast<WordCode>(i), "letter " << i << " mismatch");
    PASS();
}

void test_empty_guess_list() {
    TEST("empty guess list returns empty vector");
    auto v = load_words("nonexistent.txt");
    CHECK_EQ(v.size(), static_cast<size_t>(0), "should be empty");
    PASS();
}

int main() {
    std::ios::sync_with_stdio(false);
    std::cout << "=== Wordle Solver Tests ===\n" << std::endl;

    auto answers = load_words("data/answers.txt");
    auto allowed = load_words("data/allowed.txt");
    if (answers.empty() || allowed.empty()) {
        std::cerr << "Failed to load word lists\n";
        return 1;
    }
    auto guesses = allowed;
    guesses.insert(guesses.end(), answers.begin(), answers.end());

    auto matrix = build_pattern_matrix(guesses, answers);

    TreeBuilder builder(answers, guesses, matrix);
    auto tree = builder.build();
    const auto& pattern_cache = builder.pattern_cache();

    int na = static_cast<int>(answers.size());
    int ng = static_cast<int>(guesses.size());

    std::cout << "Data: " << na << " answers, "
              << ng << " guesses, "
              << tree.size() << " tree nodes\n" << std::endl;

    test_i32_repr_bits();
    test_empty_guess_list();
    test_encode_decode();
    test_compute_pattern();
    test_precompute_plogp();
    test_score_histogram();
    test_tie_breaker();
    test_build_histogram(matrix, na, guesses);
    test_matrix_dimensions(matrix, guesses, answers);

    test_matrix_and_cache_coherence(matrix, pattern_cache, na, tree);
    test_no_invalid_child_indices(tree);
    test_first_guess_is_soare(guesses, tree);
    test_traces(answers, tree, pattern_cache);

    test_solve_one_direct(tree, pattern_cache, answers);

    std::cout << std::endl;

    auto result = run_benchmark(tree, pattern_cache, na);

    [&]{
        TEST("worst case");
        if (result.worst_guesses != 6) { FAIL("worst case is " << result.worst_guesses); return; }
        PASS();
    }();

    [&]{
        TEST("average within range");
        if (result.avg_guesses < 3.4 || result.avg_guesses > 3.6) { FAIL("avg " << result.avg_guesses << " out of range"); return; }
        PASS();
    }();

    [&]{
        TEST("stddev within range");
        if (result.stddev_guesses < 0.5 || result.stddev_guesses > 0.7) { FAIL("stddev " << result.stddev_guesses << " out of range"); return; }
        PASS();
    }();

    test_all_words_solved(tree, pattern_cache, na);

    std::cout << "\n=== Results: "
              << passed << " passed, " << failed << " failed, "
              << (passed + failed) << " total ===" << std::endl;

    return failed > 0 ? 1 : 0;
}
