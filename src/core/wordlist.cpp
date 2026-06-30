#include "wordlist.hpp"
#include "encode.hpp"
#include <fstream>
#include <iostream>
#include <string>

std::vector<WordCode> load_words(const std::string& path) {
    std::vector<WordCode> words;
    std::ifstream file(path);
    if (!file) {
        std::cerr << "Error: cannot open " << path << "\n";
        return words;
    }
    std::string line;
    while (file >> line) {
        if (line.size() == WORD_LEN)
            words.push_back(encode_word(line));
    }
    return words;
}
