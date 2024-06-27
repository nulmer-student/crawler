#include "Include.h"

#include <filesystem>
#include <iostream>

using namespace std;
namespace Miner {

Include::Include(string input) {
    // Set the type based on the first character
    switch (input[0]) {
        case '<':
            this->type = IncludeType::System;
            break;
        case '"':
            this->type = IncludeType::User;
            break;
    }

    // Copy over the path
    string acc;
    for (int i = 1; i < input.size() - 1; i++) {
        acc.push_back(input[i]);
    }
    filesystem::path full = acc;

    // Normalize the path (remove .. & .)
    for (auto i = full.begin(); i != full.end(); i++) {
        string str = i->string();
        if (str != ".." && str != ".")
            this->path /= *i;
    }
}

} // namespace Miner
