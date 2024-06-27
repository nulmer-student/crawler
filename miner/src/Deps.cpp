#include "Deps.h"
#include "Include.h"
#include "Util.h"

#include <format>
#include <iostream>
#include <filesystem>
#include <unordered_map>
#include <vector>

using namespace std;

namespace Miner {

// =============================================================================
// Abbrev Table:
// =============================================================================

void DepGraph::print_abbrev() {
    for (auto i = this->abbrev.begin(); i != this->abbrev.end(); i++) {
        cout << "Abbrev: " << i->first << "\n";
        for (auto path : i->second) {
            cout << "  " << path.path << "\n";
        }
    }
}

void DepGraph::insert_short_path(filesystem::path k, Node v) {
    // If this abbrev exists, add the new one to the list
    if (this->abbrev.find(k) != this->abbrev.end()) {
        this->abbrev[k].push_back(v);
    }

    // Otherwise, create a new list
    else {
        this->abbrev.insert({k, {v}});
    }
}

void DepGraph::compute_abbrev() {
    for (auto i = this->nodes.begin(); i != this->nodes.end(); i++) {
        // Make path relative to the repo
        File f = i->second;
        filesystem::path path = filesystem::relative(f.path, this->repo_dir);

        // Split & reverse the path
        vector<filesystem::path> rev;
        for (auto ii = path.begin(); ii != path.end(); ii++) {
            rev.push_back(*ii);
        }

        // Add possible short paths
        filesystem::path current = rev[rev.size() - 1];
        this->insert_short_path(current, f);

        for (int ii = rev.size() - 2; ii >= 0; ii--) {
            current = rev[ii] / current;
            this->insert_short_path(current, f);
        }
    }
}

// =============================================================================
// Dependency Graph:
// =============================================================================

DepGraph::DepGraph(filesystem::path dir) {
    // Start with empty nodes & edges
    Nodes nodes;
    Edges edges;

    // Empty abbrev table
    Abbrev abbrev;
    this->repo_dir = dir;
}

void DepGraph::insert_files(vector<Key> files) {
    for (Key k : files) {
        this->insert_node(k, File(k));
    }
}

void DepGraph::insert_node(Key k, Node n) {
    this->nodes.insert({k, n});
}

void DepGraph::insert_edge(Key f1, Key f2, Include inc) {
    // TODO: If the edge is already present, don't insert

    // If there is no map with this key, insert an empty one
    if (this->edges.find(f1) == this->edges.end()) {
        this->edges.insert({f1, IncMap{}});
    }

    // Insert the edge
    this->edges[f1].insert({inc, f2});
}

void DepGraph::print_graph() {
    // Print out the nodes
    for (auto i = this->nodes.begin(); i != this->nodes.end(); i++) {
        cout << "Node: "
             << i->first.string()
             << "\n";
    }

    // Print out the edges
    for (auto i = this->edges.begin(); i != this->edges.end(); i++) {
        cout << "From: " << i->first.string() << "\n";

        // For each include, print all possibilities
        IncMap includes = i->second;
        decltype(includes.equal_range(Include("<>"))) range; // Why?
        for (auto ii = includes.begin(); ii != includes.end(); ii = range.second) {
            // Print the short header path
            cout << "  " << ii->first.path << "\n";

            // Print the range of possible paths
            range = includes.equal_range(ii->first);
            for (auto iii = range.first; iii != range.second; ++iii) {
                cout << "    " << iii->second.string() << "\n";
            }
        }
    }
}

void DepGraph::compute_dependencies() {
    // Map possible short names to headers
    this->compute_abbrev();

    // For each file, add edges according to include declarations
    for (auto i = this->nodes.begin(); i != this->nodes.end(); i++) {
        // Get the includes
        File f = i->second;
        vector<Include> includes = find_includes(f.path);

        for (auto inc : includes) {
            // Skip headers that are not in the abbrev table
            if (this->abbrev.find(inc.path) == this->abbrev.end())
                continue;

            // Add an edge to each possible full path
            vector<File> possible = this->abbrev[inc.path];
            for (auto p : possible) {
                this->insert_edge(f.path, p.path, inc);
            }
        }
    }
}

void DepGraph::naive_deps(Key current, Include inc, KeySet *found) {
    found->insert(KeyInc(current, inc));

    if (this->edges.find(current) != this->edges.end()) {
        // Find the direct dependencies
        IncMap deps = this->edges[current];
        decltype(deps.equal_range(Include("<>"))) range; // Why?

        // For each short header path, explore the first
        for (auto i = deps.begin(); i != deps.end(); i = range.second) {
            range = deps.equal_range(i->first);

            // Only check the first possibility
            if (found->contains(KeyInc(i->second, i->first)))
                continue;

            // Search the children
            naive_deps(i->second, i->first, found);
        }
    }
}

int DepGraph::path_length(Key path) {
    int count = 0;

    for (auto p : path) {
        if (p != Key("/"))
            count += 1;
    }

    return count;
}

DepGraph::Keys DepGraph::find_dirs(KeySet *dirs) {
    Keys acc;

    // Add each directory to the set
    for (auto d : *dirs) {
        // Don't include system headers
        if (d.inc.type == IncludeType::System)
            continue;

        // Don't include null includes
        if (d.inc == Include("<>"))
            continue;

        // Find the correct include directory
        int full = path_length(d.key);
        int partial = path_length(d.inc.path);
        int count = full - partial + 1;

        Key path = Key();
        for (auto p : d.key) {
            if (count <= 0)
                break;

            path /= p;
            count -= 1;
        }

        // Add the directory
        acc.insert(path);
    }

    return acc;
}

void DepGraph::compile_all() {
    int file_count = 0;
    int error_count = 0;


    // Copy the nodes into a vector for OpenMP
    vector<pair<Key, Node>> node_vec;
    for (auto i = this->nodes.begin(); i != this->nodes.end(); i++) {
        if (i->second.path.string().back() != 'c')
            continue;
        node_vec.push_back(*i);
    }

    #pragma omp parallel for
    for (int i = 0; i < node_vec.size(); i++) {
        Key key = node_vec[i].first;
        Node file = node_vec[i].second;

        if (file.path.string().back() != 'c')
            continue;

        // Find dependencies
        cout << "Processing file: " << file.path << "\n";
        file_count += 1;
        KeySet *deps = new KeySet{};
        naive_deps(key, Include("<>"), deps);

        // Get the include directories
        Keys dirs = find_dirs(deps);

        // Format includes
        string includes = "";
        for (auto ii = dirs.begin(); ii != dirs.end(); ii++) {
            includes += "-I" + ii->string() + " ";
        }

        // Compile the file
        string command = format(
            "clang -c {} {} -emit-llvm -o -",
            file.path.string(),
            includes);

        auto result = run_process(command);
        if (result.second != 0) {
            error_count += 1;
            cout << "failed" << "\n";
        } else {
            cout << "success" << "\n";
        }

        delete deps;
    }

    // Print statistics
    cout << "Total files: " << file_count << "\n";
    cout << "# Errors:    " << error_count
         << " (" << static_cast<float>(error_count) / file_count * 100.0 << "%)\n";
}

}
