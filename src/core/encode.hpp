#pragma once
#include "types.hpp"
#include <string>
#include <string_view>

WordCode encode_word(std::string_view s);
std::string decode_word(WordCode code);
