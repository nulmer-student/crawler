#include "Util.h"
#include "Compile.h"
#include "Deps.h"

#include <format>
#include <iostream>
#include <regex>
#include <sstream>
#include <vector>

#define CLANG_PATH "/home/nju/.opt/scalar/llvm-bin/bin/clang"

namespace Miner {

void compile_all(DepGraph dg) {
    // Store statistics
    int file_count = 0;
    int error_count = 0;

    // Copy the nodes into a vector for OpenMP
    vector<pair<Key, Node>> node_vec;
    for (auto i = dg.nodes.begin(); i != dg.nodes.end(); i++) {
        // Only copy cc files
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
    // Initialize the choice stack
    this->stack = vector<Action *>{};
    this->push(new Start(this->root));

    // Get the include directories
    KeySet *deps = new KeySet{};
    dg->naive_deps(this->root.path, Include("<>"), deps);
    Keys dirs = dg->find_dirs(deps);

    // Compile the file
    CompileResult result = compile_one(this->root, dirs);
    delete deps;

    return result;
}

Action *Compiler::pop() {
    Action *last = this->peek();
    this->stack.pop_back();
    return last;
}

Action *Compiler::peek() {
    return this->stack.back();
}

void Compiler::push(Action *action) {
    this->stack.push_back(action);
}

CompileResult Compiler::compile_one(Node file, Keys includes) {
    // Find dependencies
    string output = "";
    output += format("Processing file: {}\n", file.path.string());

    // Format includes
    string str_includes = "";
    for (auto ii = includes.begin(); ii != includes.end(); ii++) {
        str_includes += "-I" + ii->string() + " ";
    }

    // Compile the file
    string command = format(
        "{} -c {} {} -o /dev/null -emit-llvm -O3 -Rpass=loop-vectorize",
        CLANG_PATH,
        file.path.string(),
        str_includes);

    output += command;
    output += "\n";
    ProcessResult result = run_process(command);
    output += result.stdout;
    // output += result.stderr;

    // Parse stderr to find vectorization opportunities
    vector<Match> rem = parse_remarks(result.stderr);

    for (auto r : rem) {
        output += r.str();
        output += "\n";
    }

    // Set based on compilation pass/fail
    auto success = true;
    if (result.exit_code != 0) {
        output += format("failed\n");
        success = false;
    } else {
        output += format("success\n");
    }

    return CompileResult(success, output);
}

vector<Match> Compiler::parse_remarks(string input) {
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
    vector<Match> acc;
    string line;
    istringstream str(input);
    while (getline(str, line)) {
        if (regex_search(line, m, pattern)) {
            acc.push_back(Match(
                this->root,
                stoi(m[1].str()),
                stoi(m[2].str()),
                stoi(m[3].str()),
                stoi(m[4].str()),
                stoi(m[5].str())
                ));
        }
    }

    return acc;
}


} // namespace Miner
