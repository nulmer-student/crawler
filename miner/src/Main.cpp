#include "Util.h"
#include "Deps.h"
#include "Compile.h"

#include <string>

using namespace Miner;
using namespace std;

int main(int argc, char *argv[]) {
    // Find all source files in the repository
    string path(argv[1]);
    vector<filesystem::path> cc      = find_files(path, "c");
    vector<filesystem::path> headers = find_files(path, "h");

    // Compute a dependency graph for all files
    DepGraph dg = DepGraph(path);
    dg.insert_files(cc);
    dg.insert_files(headers);
    dg.compute_dependencies();

    // Compile each cc file
    compile_all(dg);

    return 0;
}
