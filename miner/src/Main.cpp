#include "Util.h"
#include "Deps.h"

#include <iostream>
#include <string>

using namespace Miner;
using namespace std;

int main(int argc, char *argv[]) {
    string path(argv[1]);
    vector<filesystem::path> cc      = find_files(path, "c");
    vector<filesystem::path> headers = find_files(path, "h");

    DepGraph dg = DepGraph();
    dg.insert_files(cc);
    dg.insert_files(headers);

    return 0;
}
