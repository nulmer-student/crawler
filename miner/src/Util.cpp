#include "Util.h"

#include <cstdio>
#include <fcntl.h>
#include <filesystem>
#include <fstream>
#include <sstream>
#include <stdexcept>
#include <stdio.h>
#include <unistd.h>
#include <format>
#include <sstream>
#include <string>
#include <iostream>
#include <regex>

using namespace std;

namespace Miner {

string run_process(string command) {
    // Run the command
    FILE *fp = popen(command.c_str(), "r");
    if (fp == nullptr)
        std::runtime_error("Command failure");

    // Extract output
    int c;
    string acc;
    while((c = fgetc(fp)) >= 0)
        acc += c;

    pclose(fp);
    return acc;
}

vector<filesystem::path> find_files(filesystem::path dir, string extension) {
    // Find files
    string command = format("find {} -name '*.{}'", dir.string(), extension);
    string output = run_process(command);

    // Convert to a vector of strings
    string line;
    vector<filesystem::path> acc;
    stringstream stream(output);

    while(getline(stream, line, '\n')) {
        acc.push_back(filesystem::path(line));
    }

    return acc;
}

vector<string> find_includes(filesystem::path file) {
    // Load the file
    ifstream infile(file);

    if (!infile.is_open())
        throw runtime_error("File not found");

    // Look for #include on each line
    vector<string> acc;
    string line;
    regex pattern("#include ([\"<][^\">]+[\">])");
    smatch m;
    while (getline(infile, line)) {
        if (regex_search(line, m, pattern)) {
            acc.push_back(m[1]);
        }
    }

    infile.close();
    return acc;
}

}
