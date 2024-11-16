#ifndef SI_FIND_INNER_LOOPS_H
#define SI_FIND_INNER_LOOPS_H

#include "llvm/IR/DebugInfoMetadata.h"
#include "llvm/IR/PassManager.h"

#include <vector>

using namespace llvm;

namespace InnerLoop {

// =============================================================================
// Find the locations of the innermost loops:
// =============================================================================

class InnerLoopPass : public AnalysisInfoMixin<InnerLoopPass> {
public:
  using Result = std::vector<DebugLoc>;
  Result run(Module &M, ModuleAnalysisManager &);

private:
  // LLVM analysis pass boilerplate
  static AnalysisKey Key;
  friend struct AnalysisInfoMixin<InnerLoopPass>;
};

// =============================================================================
// Print the found loop locations:
// =============================================================================

class InnerLoopPassPrinter : public PassInfoMixin<InnerLoopPassPrinter> {
public:
  PreservedAnalyses run(Module &M, ModuleAnalysisManager &);
  // This pass must be run
  static bool isRequired() { return true; }
};

} // namespace llvm

#endif
