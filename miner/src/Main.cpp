#include "Util.h"
#include "Deps.h"
#include "Compile.h"
#include "Config.h"

#include <csignal>
#include <cstdio>
#include <string>
#include <execinfo.h>
#include <signal.h>
#include <unistd.h>
#include <omp.h>

using namespace Miner;
using namespace std;

void handler(int sig) {
    // Get stackframes
    void *array[10];
    int size = backtrace(array, 10);

    // Print out stackframes
    fprintf(stderr, "Error: singal %d\n", sig);
    backtrace_symbols_fd(array, size, STDERR_FILENO);
    exit(1);
}

int main(int argc, char **argv) {
    // Register signal handler
    signal(SIGSEGV, handler);

    // Handle arguments
    struct arguments args;
    args.threads = 12;
    args.log = "./log";

    argp_parse(&arg_p, argc, argv, 0, 0, &args);

    // Set the number of threads to use
    omp_set_num_threads(args.threads);

    // Find all source files in the repository
    string path(args.args[0]);
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
