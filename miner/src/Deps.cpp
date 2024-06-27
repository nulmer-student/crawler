#include "Deps.h"

#include <filesystem>
#include <vector>

using namespace std;

namespace Miner {

DepGraph::DepGraph() {
    // Start with empty nodes & edges
    unordered_map<Key, Node, KeyHash, KeyEq> nodes;
    unordered_map<Key, vector<Edge>, KeyHash, KeyEq> edges;
}

void DepGraph::insert_files(vector<Key> files) {
    for (Key k : files) {
        this->insert_node(k, File(k));
    }
}


void DepGraph::insert_node(Key k, Node n) {
    this->nodes.insert({k, n});
}

void DepGraph::compute_dependencies() {
    // TODO: Add dependencies between files
}

}
