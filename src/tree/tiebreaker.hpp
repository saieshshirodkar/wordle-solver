#pragma once
#include <cstdint>

bool tie_breaker(const uint16_t* new_hist, const uint16_t* old_hist,
                 bool new_is_answer, bool old_is_answer);
