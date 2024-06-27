#ifndef DEPS_H_
#define DEPS_H_

#include "Util.h"

#include <cstddef>
#include <filesystem>
#include <unordered_map>

using namespace std;

namespace Miner {

// =============================================================================
// Nodes & Edges:
// =============================================================================

class Edge {
public:
    string name;
};

class File {
public:
    File(filesystem::path p) : path(p){};
    filesystem::path path;
};

typedef filesystem::path Key;
typedef File Node;

// =============================================================================
// Dependency Graph:
// =============================================================================

struct KeyHash {
    size_t operator()(const Key &k) const {
        return hash<string>{}(k.string());
    }
};

struct KeyEq {
    size_t operator()(const Key &a, const Key &b) const {
        return a.string() == b.string();
    }
};

class DepGraph {
public:
    DepGraph(filesystem::path);
    void insert_files(vector<Key>);
    void insert_node(Key, Node);

    void compute_dependencies();
    void print_abbrev();

private:
    void compute_abbrev();
    void insert_short_path(filesystem::path, Node);

    // Nodes are repository files
    unordered_map<Key, Node, KeyHash, KeyEq> nodes;

    // Edges are #include X declarations
    unordered_map<Key, vector<Edge>, KeyHash, KeyEq> edges;

    // Map short include paths (eg #include for/bar.h) to full paths
    unordered_map<filesystem::path, vector<File>> abbrev;
    filesystem::path repo_dir;
};

}

#endif // DEPS_H_
