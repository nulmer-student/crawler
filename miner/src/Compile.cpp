#include "Util.h"
#include "Compile.h"
#include "Deps.h"

#include <format>
#include <iostream>
#include <regex>
#include <sstream>

#define CLANG_PATH "/home/nju/.opt/scalar/llvm-bin/bin/clang"

namespace Miner {

void compile_all(DepGraph dg) {
    // Store statistics
    int file_count = 0;
    int error_count = 0;

    // Copy the nodes into a vector for OpenMP
    vector<pair<Key, Node>> node_vec;
    for (auto i = dg.nodes.begin(); i != dg.nodes.end(); i++) {
        if (i->second.path.string().back() != 'c')
            continue;
        node_vec.push_back(*i);
    }

    #pragma omp parallel for
    for (int i = 0; i < node_vec.size(); i++) {
        #pragma omp atomic
        file_count += 1;

        // Compile the file
        Compiler c = Compiler(&dg, node_vec[i].second);
        CompileResult result = c.run();
        cout << result.output;

        // Update statistics
        if (!result.success)
            #pragma omp atomic
            error_count += 1;
    }

    // Print statistics
    float prop = static_cast<float>(error_count) / file_count * 100.0;
    cout << format("Total files: {:5}\n", file_count);
    cout << format("Successful:  {:5} ({:5.1f}%)\n", file_count - error_count, 100.0 - prop);
    cout << format("Errors:      {:5} ({:5.1f}%)\n", error_count, prop);
}

// =============================================================================
// Compiler
// =============================================================================

CompileResult Compiler::run() {
    // return CompileResult(true, "");
    return compile_one(this->root);
}

CompileResult Compiler::compile_one(Node file) {
    string output = "";

    // Find dependencies
    output += format("Processing file: {}\n", file.path.string());
    KeySet *deps = new KeySet{};
    dg->naive_deps(file.path, Include("<>"), deps);

    // Get the include directories
    Keys dirs = dg->find_dirs(deps);

    // Format includes
    string includes = "";
    for (auto ii = dirs.begin(); ii != dirs.end(); ii++) {
        includes += "-I" + ii->string() + " ";
    }

    // Compile the file
    string command = format(
        "{} -c {} {} -o /dev/null -emit-llvm -O3 -Rpass=loop-vectorize",
        CLANG_PATH,
        file.path.string(),
        includes);

    ProcessResult result = run_process(command);
    output += result.stdout;

    // Parse stderr to find vectorization opportunities
    string rem = parse_remarks(result.stderr);
    if (rem != "") {
        output += "line, column, width, interleave, scalar\n";
        output += rem;
    }

    // Set based on compilation pass/fail
    auto success = true;
    if (result.exit_code != 0) {
        output += format("failed\n");
        success = false;
    } else {
        output += format("success\n");
    }

    delete deps;
    return CompileResult(success, output);
}

string Compiler::parse_remarks(string input) {
    // Construct the pattern
    // FIXME: Make less fragile
    string pat = "";
    pat += "(\\d+):(\\d+): ";
    pat += "remark: vectorized loop \\(";
    pat += "vectorization width: (\\d+),";
    pat += " interleaved count: (\\d+),";
    pat += " scalar interpolation count: (\\d+)";
    pat += "\\)";

    regex pattern(pat);
    smatch m;

    // Search each line in the input for the pattern
    string found = "";
    string line;
    istringstream str(input);
    while (getline(str, line)) {
        if (regex_search(line, m, pattern)) {
            found += format(
                "{}, {}, {}, {}, {}\n",
                m[1].str(), m[2].str(),
                m[3].str(), m[4].str(), m[5].str());
        }
    }

    return found;
}


} // namespace Miner
