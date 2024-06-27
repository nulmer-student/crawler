#include "Util.h"

#include <cstdio>
#include <fcntl.h>
#include <filesystem>
#include <sstream>
#include <stdexcept>
#include <stdio.h>
#include <unistd.h>
#include <format>
#include <sstream>
#include <string>

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

vector<string> find_files(filesystem::path dir, string extension) {
    // Find files
    string command = format("find {} -name '*.{}'", dir.string(), extension);
    string output = run_process(command);

    // Convert to a vector of files
    string line;
    vector<string> acc;
    stringstream stream(output);

    while(getline(stream, line, '\n')) {
        acc.push_back(line);
    }

    return acc;
}
