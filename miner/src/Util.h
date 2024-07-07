#ifndef CRAWLER_UTIL_H
#define CRAWLER_UTIL_H

#include "Include.h"

#include <filesystem>
#include <iostream>
#include <streambuf>
#include <string>
#include <vector>

using namespace std;

namespace Miner {

// Hold the results of running a process
struct ProcessResult {
    int exit_code;
    string stdout;
    string stderr;
};

ProcessResult test_run(string command, string stdin);

// Utility functions
ProcessResult run_process(string command);
vector<filesystem::path> find_files(filesystem::path dir, string extension);
vector<Include> find_includes(filesystem::path file);

}

#endif
