#ifndef SI_INFORMATION_H
#define SI_INFORMATION_H

#include "llvm/IR/PassManager.h"
#include "llvm/IR/Function.h"
#include "llvm/Analysis/LoopInfo.h"

#include <llvm/IR/DebugLoc.h>
#include <optional>
#include <vector>
#include <set>

using namespace std;
using namespace llvm;

namespace Info {

// =============================================================================
// Find the required information:
// =============================================================================

// Instruction mix:

class IRMix {
public:
  // Start with counts zero initialized
  IRMix() : count(0), mem_count(0), arith_count(0), other_count(0){};

  int count;        // Total number of instructions
  int mem_count;    // Number of memory instructions
  int arith_count;  // Number of arithmetic instructions
  int other_count;  // All other types of instructions
};

// Memory pattern:

class MemPattern {
public:
  MemPattern() : start(nullopt), step(nullopt){};

  optional<int> start; // Initial value of the IV
  optional<int> step;  // Step of the IV
};

// Overall result

class InfoData {
public:
  InfoData(set<int> locations, IRMix mix, MemPattern pattern)
    : locations(locations), mix(mix), pattern(pattern){};

  set<int> locations;
  IRMix mix;
  MemPattern pattern;

  string to_string();
};

class InfoPass : public AnalysisInfoMixin<InfoPass> {
public:
  using Result = vector<InfoData>;
  Result run(Function &F, FunctionAnalysisManager &);

private:
  // LLVM analysis pass boilerplate
  static AnalysisKey Key;
  friend struct AnalysisInfoMixin<InfoPass>;

  // Collect the locations of every instruction in a loop
  set<int> collect_locations(Loop *loop);

  // Compute the instruction mix of a loop
  IRMix find_ir_mix(Loop *loop);

  // Compute the memory access patterns of a loop
  MemPattern find_mem_pattern(Loop *loop, FunctionAnalysisManager &FAM);
};

// =============================================================================
// Print the found information:
// =============================================================================

class InfoPassPrinter : public PassInfoMixin<InfoPassPrinter> {
public:
  PreservedAnalyses run(Function &F, FunctionAnalysisManager &);
  // This pass must be run
  static bool isRequired() { return true; }
};

} // namespace llvm

#endif // INFORMATION_H_
