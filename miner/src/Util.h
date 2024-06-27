#ifndef CRAWLER_UTIL_H
#define CRAWLER_UTIL_H

#include <filesystem>
#include <string>
#include <vector>

using namespace std;

string run_process(string command);
vector<string> find_files(filesystem::path dir, string extension);

#endif
