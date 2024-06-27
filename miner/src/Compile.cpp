#include "Util.h"
#include "Compile.h"
#include "Deps.h"

#include <cstddef>
#include <format>
#include <iostream>
#include <iterator>
#include <regex>
#include <sstream>
#include <stdexcept>
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

    // #pragma omp parallel for
    // for (int i = 0; i < 1; i++) {
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
// Actions
// =============================================================================

string Start::str() {
    return format("Start({})", dest.path.string());
}

string Foreward::str() {
    return format(
        "Foreward({}, {})",
        src.path.string(),
        dest.path.string());
}

string Backward::str() {
    return format(
        "Backward({}, {})",
        src.path.string(),
        dest.path.string());
}

// =============================================================================
// Compiler
// =============================================================================

CompileResult Compiler::run() {
    // Initialize
    this->stack = vector<Action *>{};
    this->push(new Start(this->root));
    KeySet seen;
    this->parents = Ans{};
    KeySet *deps = new KeySet{};

    while (true) {
        cout << "Stack:\n" << this->dump_stack() << "\n";

        // Get the current location
        Action *action = this->peek();
        Node current = action->dest;
        Key path = current.path;
        cout << path << "\n";

        // Get the children
        bool any = false;
        if (dg->edges.find(path) != dg->edges.end()) {
            // Get the direct dependencies
            DepGraph::IncMap inc = dg->edges[path];

            // Explore the dependencies
            decltype(inc.equal_range(Include("<>"))) range; // Why?

            bool none = true;
            for (auto i = inc.begin(); i != inc.end(); i = range.second) {
                range = inc.equal_range(i->first);

                // Don't check already visited nodes
                if (seen.contains(KeyInc(i->second, i->first)))
                    continue;

                // Visit the first unseen child
                any = true;
                seen.insert(KeyInc(i->second, i->first));
                this->push(new Foreward(current, File(i->second)));

                // Save the include file
                deps->insert(KeyInc(i->second, i->first));
                break;

                none = false;
            }

            // End if we are goind backward to the root & there are no more
            // children
            if (none) {
                Backward *bk = dynamic_cast<Backward *>(this->peek());
                if (bk != nullptr && bk->dest.path == this->root.path)
                    break;
            }
        }

        // If no children matched, go backward
        if (!any) {
            Node parent = this->parent(current);
            this->push(new Backward(File(current), parent));
        }
    }

    // Get the include directories
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
    // If this is a foreward action, save the parent
    Foreward *fw = dynamic_cast<Foreward *>(action);
    if (fw != nullptr) {
        this->parents.insert({fw->dest.path, fw->src.path});
    }

    // Add the action
    this->stack.push_back(action);
}

string Compiler::dump_stack() {
    string acc = "";
    for (Action *element : this->stack) {
        acc += element->str();
        acc += "\n";
    }
    return acc;
}

Node Compiler::parent(Node current) {
    if (parents.find(current.path) == parents.end()) {
        throw runtime_error(format("Missing parent: {}", current.path.string()));
    }

    return parents[current.path];
}

// =============================================================================
// Compiler a Single File
// =============================================================================

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
