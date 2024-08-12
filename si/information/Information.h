#ifndef SI_INFORMATION_H
#define SI_INFORMATION_H

#include "llvm/IR/PassManager.h"
#include "llvm/IR/Function.h"

#include <vector>
#include <unordered_set>

using namespace llvm;

namespace Info {

// =============================================================================
// Find the required information:
// =============================================================================

class IRMix {
public:
  // Start with count zero initialized
  IRMix() : mem_count(0), arith_count(0), count(0){};

  int mem_count;    // Number of memory instructions
  int arith_count;  // Number of arithmetic instructions
  int count;        // Total number of instructions
};

class InfoPass : public AnalysisInfoMixin<InfoPass> {
public:
  using Result = std::vector<DebugLoc>;
  Result run(Function &F, FunctionAnalysisManager &);

private:
  // LLVM analysis pass boilerplate
  static AnalysisKey Key;
  friend struct AnalysisInfoMixin<InfoPass>;

  // Find the locations of relevent loops
  using Locs = std::unordered_set<int>;
  Locs parse_loop_locs();

  // Compute the instruction mix of a loop
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
