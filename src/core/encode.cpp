#include "encode.hpp"

WordCode encode_word(std::string_view s) {
    WordCode code = 0;
    for (std::size_t i = 0; i < WORD_LEN; i++)
        code |= static_cast<WordCode>(static_cast<unsigned char>(s[i] - 'a')) << (i * 5);
    return code;
}

std::string decode_word(WordCode code) {
    std::string s(WORD_LEN, ' ');
    for (std::size_t i = 0; i < WORD_LEN; i++)
        s[i] = static_cast<char>('a' + ((code >> (i * 5)) & 0x1F));
    return s;
}
