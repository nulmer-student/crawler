#include "PreProcess.h"

#include <regex>
#include <iostream>
#include <fstream>
#include <stdexcept>

using namespace std;

namespace Miner {

string insert_pragma(filesystem::path path) {
    string pragma = "#pragma clang loop scalar_interpolation(enable)\n";
    string acc = "";

    // Open the file
    ifstream file(path);
    if (!file.is_open())
        throw std::runtime_error("Failed to open file");

    // Insert a pragma before each for loop
    string line;
    regex pattern("\\s*for\\s*\\(");
    smatch match;
    while (getline(file, line)) {
        // If this line starts a for loop, insert the pragma
        if (regex_search(line, match, pattern)) {
            acc += pragma;
        }

        acc += line;
        acc += "\n";
    }

    file.close();

    return acc;
}

}
