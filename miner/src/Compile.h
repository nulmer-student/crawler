#ifndef COMPILE_H_
#define COMPILE_H_

#include "Deps.h"

using namespace std;
namespace Miner {

// Compile all cc files
void compile_all(DepGraph dg);

// =============================================================================
// Complier
// =============================================================================

// Result of a single compilation
struct CompileResult {
    bool success;
    string output;
};

class Action {
public:
    Action(Node file) : file(file){};
    Node file;
};

class Optional : Action {};
class Required : Action {};

// Compiles a single file, checking all possible header choices:
class Compiler {
public:
    Compiler(DepGraph *dg, Node root) : dg(dg), root(root){};
    CompileResult run();

private:
    DepGraph *dg;   // File dependency graph
    Node root;      // File that we are compiling

    // When searching, store the choices we have made
    vector<Action*> choice_stack;

    // Complie a single file
    CompileResult compile_one(Node file);

    // Extract vectorization opportunities
    string parse_remarks(string input);
};

}

#endif // COMPILE_H_
