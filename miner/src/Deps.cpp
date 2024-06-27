#include "Deps.h"
#include "Util.h"

#include <iostream>
#include <filesystem>
#include <vector>

using namespace std;

namespace Miner {

// =============================================================================
// Abbreviation Table:
// =============================================================================



// =============================================================================
// Dependency Graph:
// =============================================================================

DepGraph::DepGraph(filesystem::path dir) {
    // Start with empty nodes & edges
    unordered_map<Key, Node, KeyHash, KeyEq> nodes;
    unordered_map<Key, vector<Edge>, KeyHash, KeyEq> edges;

    // Empty abbrev table
    unordered_map<filesystem::path, vector<File>> abbrev;
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

void DepGraph::print_abbrev() {
    for (auto i = this->abbrev.begin(); i != this->abbrev.end(); i++) {
        cout << "Abbrev: " << i->first << "\n";
        for (auto path : i->second) {
            cout << "  " << path.path << "\n";
        }
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

void DepGraph::compute_dependencies() {
    // Map possible short names to headers
    this->compute_abbrev();
    this->print_abbrev();

    // for (auto it = this->nodes.begin(); it != this->nodes.end(); it++) {
    //     // Get the includes
    //     File f = it->second;
    //     vector<string> includes = find_includes(f.path);
    // }
}

}
