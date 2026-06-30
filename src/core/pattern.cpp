#include "pattern.hpp"

static void count_letters(WordCode word, uint8_t* counts) {
    for (int i = 0; i < WORD_LEN; i++) {
        counts[word & 0x1F]++;
        word >>= 5;
    }
}

Pattern compute_pattern(WordCode guess, WordCode target) {
    uint8_t counts[NUM_LETTERS] = {0};
    count_letters(target, counts);

    Pattern pat = 0;

    for (int i = 0; i < WORD_LEN; i++) {
        int g = (guess >> (i * 5)) & 0x1F;
        int a = (target >> (i * 5)) & 0x1F;
        if (g == a) {
            pat = static_cast<Pattern>(pat + 2 * POW3[i]);
            counts[g]--;
        }
    }

    for (int i = 0; i < WORD_LEN; i++) {
        int g = (guess >> (i * 5)) & 0x1F;
        int a = (target >> (i * 5)) & 0x1F;
        if (g != a && counts[g] > 0) {
            pat = static_cast<Pattern>(pat + 1 * POW3[i]);
            counts[g]--;
        }
    }

    return pat;
}
