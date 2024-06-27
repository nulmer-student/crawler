#ifndef DEPS_H_
#define DEPS_H_

#include "Include.h"

#include <cstddef>
#include <filesystem>
#include <unordered_map>
#include <unordered_set>
#include <vector>

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

    void insert_edge(Key f1, Key f2, Include inc);

    void compute_dependencies();
    void print_abbrev();
    void print_graph();

private:
    void compute_abbrev();
    void insert_short_path(filesystem::path, Node);

    using KeySet = unordered_set<Key, KeyHash, KeyEq>;
    void naive_deps(Key current, KeySet *found);

    // Nodes are repository files
    using Nodes = unordered_map<Key, Node, KeyHash, KeyEq>;
    Nodes nodes;

    // Edges are #include X declarations
    using IncMap = unordered_multimap<Include, Key, IncludeHash, IncludeEq>;
    using Edges  = unordered_map<Key, IncMap, KeyHash, KeyEq>;
    Edges edges;

    // Map short include paths (eg #include for/bar.h) to full paths
    using Abbrev = unordered_map<filesystem::path, vector<File>>;
    Abbrev abbrev;
    filesystem::path repo_dir;
};

}

#endif // DEPS_H_
