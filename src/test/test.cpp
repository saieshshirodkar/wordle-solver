#include "../core/wordlist.hpp"
#include "../core/pattern.hpp"
#include "../core/matrix.hpp"
#include "../core/encode.hpp"
#include "../tree/builder.hpp"
#include "../solver/benchmark.hpp"
#include <iostream>
#include <string>
#include <cassert>
#include <cmath>
#include <algorithm>

static int passed = 0, failed = 0;

#define TEST(name) do { std::cout << "  " << name << "... "; } while(0)
#define PASS() do { std::cout << "PASS\n"; passed++; } while(0)
#define FAIL(msg) do { std::cout << "FAIL: " << msg << "\n"; failed++; } while(0)
#define CHECK(cond, msg) do { if (!(cond)) { FAIL(msg); return; } } while(0)
#define CHECK_EQ(a, b, msg) do { if ((a) != (b)) { FAIL(msg << " (" << (a) << " != " << (b) << ")"); return; } } while(0)

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

void test_known_trace(const std::vector<WordCode>& answers,
                      const std::vector<Node>& tree,
                      const std::vector<uint8_t>& pattern_cache,
                      const std::string& word, int expected_guesses) {
    WordCode code = encode_word(word);
    int secret = -1;
    for (int i = 0; i < (int)answers.size(); i++) {
        if (answers[i] == code) { secret = i; break; }
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

    std::cout << "Data: " << answers.size() << " answers, "
              << guesses.size() << " guesses, "
              << tree.size() << " tree nodes\n" << std::endl;

    test_encode_decode();
    test_compute_pattern();

    test_matrix_and_cache_coherence(matrix, pattern_cache,
                                     (int)answers.size(), tree);
    test_no_invalid_child_indices(tree);
    test_first_guess_is_soare(guesses, tree);
    test_traces(answers, tree, pattern_cache);

    std::cout << std::endl;

    auto result = run_benchmark(tree, pattern_cache, static_cast<int>(answers.size()));

    {
        TEST("worst case");
        bool ok = (result.worst_guesses == 6);
        if (ok) { PASS(); }
        else { FAIL("worst case is " << result.worst_guesses << ", expected 6"); }
    }

    {
        TEST("average within range");
        bool ok = (result.avg_guesses >= 3.4 && result.avg_guesses <= 3.6);
        if (ok) { PASS(); }
        else { FAIL("avg " << result.avg_guesses << " out of range [3.4, 3.6]"); }
    }

    test_all_words_solved(tree, pattern_cache, (int)answers.size());

    std::cout << "\n=== Results: "
              << passed << " passed, " << failed << " failed, "
              << (passed + failed) << " total ===" << std::endl;

    return failed > 0 ? 1 : 0;
}
