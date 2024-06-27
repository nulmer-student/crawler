#include "Util.h"
#include "Compile.h"
#include "Deps.h"

#include <format>
#include <iostream>

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

    // Compile each file
    #pragma omp parallel for
    for (int i = 0; i < node_vec.size(); i++) {
        string output = "";

        Key key = node_vec[i].first;
        Node file = node_vec[i].second;

        if (file.path.string().back() != 'c')
            continue;

        // Find dependencies
        output += format("Processing file: {}\n", file.path.string());
        file_count += 1;
        KeySet *deps = new KeySet{};
        dg.naive_deps(key, Include("<>"), deps);

        // Get the include directories
        Keys dirs = dg.find_dirs(deps);

        // Format includes
        string includes = "";
        for (auto ii = dirs.begin(); ii != dirs.end(); ii++) {
            includes += "-I" + ii->string() + " ";
        }

        // Compile the file
        string command = format(
            "clang -c {} {} -emit-llvm -o - 2> /dev/null",
            file.path.string(),
            includes);

        auto result = run_process(command);
        if (result.second != 0) {
            error_count += 1;
            output += format("failed\n");
        } else {
            output += format("success\n");
        }

        delete deps;
        cout << output;
    }

    // Print statistics
    float prop = static_cast<float>(error_count) / file_count * 100.0;
    cout << format("Total files: {:5}\n", file_count);
    cout << format("Successful:  {:5} ({:5.1f}%)\n", file_count - error_count, 100.0 - prop);
    cout << format("Errors:      {:5} ({:5.1f}%)\n", error_count, prop);
}

} // namespace Miner
