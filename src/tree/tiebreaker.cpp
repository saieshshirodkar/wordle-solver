#include "tiebreaker.hpp"

bool tie_breaker(const uint16_t* new_hist, const uint16_t* old_hist,
                 bool new_is_answer, bool old_is_answer) {
    if (new_is_answer != old_is_answer)
        return new_is_answer;

    uint16_t new_max = 0, old_max = 0;
    int new_nonzero = 0, old_nonzero = 0;

    for (int p = 0; p < 243; p++) {
        if (new_hist[p] > 0) {
            new_nonzero++;
            if (new_hist[p] > new_max) new_max = new_hist[p];
        }
        if (old_hist[p] > 0) {
            old_nonzero++;
            if (old_hist[p] > old_max) old_max = old_hist[p];
        }
    }

    if (new_max != old_max)
        return new_max < old_max;

    return new_nonzero > old_nonzero;
}
