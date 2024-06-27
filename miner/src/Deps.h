#ifndef DEPS_H_
#define DEPS_H_

#include "Include.h"
#include "Util.h"

#include <cstddef>
#include <filesystem>
#include <unordered_map>
#include <utility>
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

    // Graph data
    using Nodes = unordered_map<Key, Node, KeyHash, KeyEq>;
    using Edges = unordered_map<Key, vector<pair<Key, Include>>>;
    Nodes nodes;    // Nodes are repository files
    Edges edges;    // Edges are #include X declarations

    // Map short include paths (eg #include for/bar.h) to full paths
    using Abbrev = unordered_map<filesystem::path, vector<File>>;
    Abbrev abbrev;
    filesystem::path repo_dir;
};

}

#endif // DEPS_H_
