#ifndef COMPILE_H_
#define COMPILE_H_

#include "Deps.h"

using namespace std;
namespace Miner {

struct CompileResult {
    bool success;
};

// Compiles a single file, checking all possible header choices
class Compiler {
public:
    Compiler(DepGraph *dg) : dg(dg){};

    CompileResult compile_one(Node file);

private:
    DepGraph *dg;
};

void compile_all(DepGraph dg);

}

#endif // COMPILE_H_
