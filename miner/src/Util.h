#ifndef CRAWLER_UTIL_H
#define CRAWLER_UTIL_H

#include <filesystem>
#include <string>
#include <vector>

using namespace std;

namespace Miner {

string run_process(string command);
vector<filesystem::path> find_files(filesystem::path dir, string extension);
vector<string> find_includes(filesystem::path file);

}

#endif
