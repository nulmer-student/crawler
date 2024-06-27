#include "Deps.h"
#include "Util.h"

#include <iostream>
#include <filesystem>
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
    // It it exists, add the edge to the list of edges
    if (this->edges.find(f1) != this->edges.end()) {
        this->edges[f1].push_back(pair(f2, inc));
    }

    // Otherwise, create a new list
    else {
        this->edges.insert({f1, {pair(f2, inc)}});
    }
}

void DepGraph::print_graph() {
    // Print out the nodes
    for (auto i = this->nodes.begin(); i != this->nodes.end(); i++) {
        cout << "Node: ("
             << i->first.string() << ", "
             << i->second.path.string()
             << ")\n";
    }

    // Print out the edges
    for (auto i = this->edges.begin(); i != this->edges.end(); i++) {
        cout << "From: " << i->first.string() << "\n";
        for (auto e : i->second) {
            cout << "  To: "
                 << e.first.string()
                 << " ("
                 << e.second.path.string()
                 << ")\n";
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

    this->print_graph();
}

}
