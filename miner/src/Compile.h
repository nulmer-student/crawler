#ifndef COMPILE_H_
#define COMPILE_H_

#include "Deps.h"
#include <format>
#include <unordered_map>
#include <vector>

using namespace std;
namespace Miner {

// Compile all cc files
void compile_all(DepGraph dg);

class Compiler;

// Result of a single compilation
struct CompileResult {
    bool success;
    string output;
};

// =============================================================================
// Actions
// =============================================================================

class Action {
public:
    Action(Node dest) : dest(dest){};
    Node dest;

    virtual string str() = 0;

    // By default to nothing on push/pop
    virtual void on_push(Compiler *cc) { return; }
    virtual void on_pop(Compiler *cc) { return; }
};

class Start : public Action {
public:
    Start(Node dest) : Action(dest){};
    virtual string str();
};

// Movement between nodes:

class Move : public Action {
public:
    Move(Node src, Node dest) : src(src), Action(dest){};
    Node src;
};

class Foreward : public Move {
public:
    Foreward(Node src, Node dest, KeyInc include)
        : Move(src, dest), include(include){};
    virtual string str();
    virtual void on_push(Compiler *cc);
    KeyInc include;
};

class Backward : public Move {
public:
    Backward(Node src, Node dest) : Move(src, dest){};
    virtual string str();
};

// Choices

class Choice : public Foreward {
public:
    Choice(Node src, Node dest, KeyInc include)
        : Foreward(src, dest, include){};
};

class Many : public Choice {
public:
    Many(Node src, Node dest, KeyInc include, vector<Node> rest)
        : Choice(src, dest, include), rest(rest){};
    virtual string str();
    vector<Node> rest;
};

// =============================================================================
// Complier
// =============================================================================

// Data associated with a single match
class Match {
public:
    Match(File file, int line, int column, int width, int interleave, int scalar)
    : file(file), line(line), column(column), width(width),
      interleave(interleave), scalar(scalar){};

    // Location info
    File file;
    int line;
    int column;

    // Match info
    int width;
    int interleave;
    int scalar;

    string str() {
        return format("{} {} {} {} {}", line, column, width, interleave, scalar);
    }
};

// Compiles a single file, checking all possible header choices:
class Compiler {
public:
    Compiler(DepGraph *dg, Node root) : dg(dg), root(root){};
    CompileResult run();

    void insert_parent(Key a, Key b);

private:
    DepGraph *dg;   // File dependency graph
    Node root;      // File that we are compiling

    // Try to complie a single file
    CompileResult compile_one(Node file, Keys includes);

    // Extract vectorization opportunities from the compiler output
    vector<Match> parse_remarks(string input);

    // When searching, store the choices we have made
    vector<Action *> stack;

    using Ans = unordered_map<Key, Key, KeyHash, KeyEq>;
    Ans parents;
    Node parent(Node current);

    void expand();
    bool shrink();

    Action *peek();
    void pop();
    void push(Action *action);
    string dump_stack();
};

}

#endif // COMPILE_H_
