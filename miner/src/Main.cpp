#include "Util.h"

#include <iostream>

using namespace std;

int main(int argc, char *argv[]) {
    vector<string> files = find_files(
        "/home/nju/downloads/tmp/repositories/36502-git",
        "h");

    for (auto f : files) {
        cout << f << "\n";
    }

    return 0;
}

