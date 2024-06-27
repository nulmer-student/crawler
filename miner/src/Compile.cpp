#include "Util.h"
#include "Compile.h"
#include "Deps.h"

#include <format>
#include <iostream>
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

string Many::str() {
    string acc = format("Many({}, {}", src.path.string(), dest.path.string());
    for (auto other : this->rest) {
        acc += ", ";
        acc += other.key.string();
    }
    acc += ")";

    return acc;
}

void Foreward::on_push(Compiler *cc) {
    cc->insert_parent(this->dest.path, this->src.path);
}

bool Many::next() {
    if (this->rest.size() == 0)
        return false;

    // Replace with a new destination
    KeyInc last = this->rest.back();
    this->dest = File(last.key);
    this->include = last;
    this->rest.pop_back();

    return true;
}

// =============================================================================
// Compiler
// =============================================================================

CompileResult Compiler::run() {
    // Initialize
    this->stack = vector<Action *>{};
    this->push(new Start(this->root));
    this->seen = Keys{};

    this->parents = Ans{};
    vector<KeyInc> include_dirs;

    CompileResult result;
    while (true) {
        // Expand the search tree from the current point
        this->expand();

        // All visited files are includes
        include_dirs.clear();
        for (auto element : this->stack) {
            Foreward *fw = dynamic_cast<Foreward *>(element);
            if (fw != nullptr)
                include_dirs.push_back(fw->include);
        }

        // FIXME: Only compile if we haven't already tried this combination

        // Try to compile the file
        Keys dirs = dg->find_dirs(include_dirs);
        result = compile_one(this->root, dirs);

        // Stop if compilation succeeds
        if (result.success)
            break;

        // Otherwise backtrack to the last choice-point
        bool cont = this->shrink();

        if (!cont)
            break;
    }

    return result;
}

void Compiler::expand() {
    while (true) {
        // Get the current location
        Action *action = this->peek();
        Node current = action->dest;
        Key path = current.path;

        // cout << path << "\n";
        // cout << "Stack:\n" << this->dump_stack() << "\n";
        // for (auto element : this->parents) {
        //     cout << "p: " << element.first << " " << element.second << "\n";
        // }
        // cout << std::flush;

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
                if (this->seen.contains(i->second))
                  continue;

                any = true;
                this->seen.insert(i->second);

                // Collect the choices
                vector<KeyInc> choices;
                for (auto choice = range.first; choice != range.second; ++choice) {
                    choices.push_back(KeyInc(choice->second, choice->first));
                }

                // If there is only one choice, insert a Foreward()
                if (choices.size() == 1) {
                    // Visit the single child
                    this->push(
                        new Foreward(
                            current,
                            File(i->second),
                            KeyInc(i->second, i->first)));
                    break;
                }

                // Otherwise, insert a Many()
                else {
                    // Select one possibility to use
                    KeyInc chosen = choices[choices.size() - 1];
                    choices.pop_back();

                    // Insert the many
                    this->push(
                        new Many(
                            current,
                            File(chosen.key),
                            chosen,
                            choices));

                    break;
                }

                none = false;
            }

            // If there are no children:
            if (none) {
                // End if we are goind backward to the root
                Backward *bk = dynamic_cast<Backward *>(this->peek());
                if (bk != nullptr && bk->dest.path == this->root.path)
                    break;
            }
        }

        // If no children matched, go backward
        if (!any) {
            // End if this is the start
            Start *start = dynamic_cast<Start *>(this->peek());
            if (start != nullptr)
              break;

            Node parent = this->parent(current);
            this->push(new Backward(File(current), parent));
        }
    }
}

bool Compiler::shrink() {
    // cout << "shrink" << "\n";

    while (true) {
        // cout << this->dump_stack() << "\n";

        Action *current = this->peek();

        // If we have reached the start, there are no more choices
        Start *start = dynamic_cast<Start *>(current);
        if (start != nullptr)
            return false;

        // If we are at a choice point, return
        Choice *choice = dynamic_cast<Choice *>(current);
        if (choice != nullptr) {
            bool more = choice->next();

            // Remove this action if there are no more possibilities
            if (!more) {
                this->pop();
                continue;
            }

            // Otherwise, try this choice
            choice->on_push(this);
            return true;
        }

        // Otherwise, remove the element
        this->seen.erase(this->peek()->dest.path);
        this->pop();
    }
}

Action *Compiler::peek() {
    return this->stack.back();
}

void Compiler::pop() {
    this->peek()->on_pop(this);
    this->stack.pop_back();
}

void Compiler::push(Action *action) {
    this->stack.push_back(action);
    this->peek()->on_push(this);
}

string Compiler::dump_stack() {
    string acc = "";
    string spc = "";
    for (Action *element : this->stack) {
        Foreward *fw = dynamic_cast<Foreward *>(element);
        if (fw != nullptr)
            spc += "  ";
        acc += spc;
        acc += element->str();
        acc += "\n";

        Backward *bw = dynamic_cast<Backward *>(element);
        if (bw != nullptr && spc.size() >= 2) {
            spc.pop_back();
            spc.pop_back();
        }

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

void Compiler::insert_parent(Key dest, Key src) {
    this->parents.insert({dest, src});
}

} // namespace Miner
