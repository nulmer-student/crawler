#ifndef CRAWLER_UTIL_H
#define CRAWLER_UTIL_H

#include "Include.h"

#include <filesystem>
#include <string>
#include <vector>

using namespace std;

namespace Miner {

pair<string, int> run_process(string command);
vector<filesystem::path> find_files(filesystem::path dir, string extension);
vector<Include> find_includes(filesystem::path file);

}

#endif
