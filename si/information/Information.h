#ifndef SI_INFORMATION_H
#define SI_INFORMATION_H

#include "llvm/IR/PassManager.h"
#include "llvm/IR/Function.h"

#include <utility>
#include <vector>

using namespace llvm;

namespace Info {

// =============================================================================
// Find the required information:
// =============================================================================

class InfoPass : public AnalysisInfoMixin<InfoPass> {
public:
  using Result = std::vector<DebugLoc>;
  Result run(Function &F, FunctionAnalysisManager &);

private:
  // LLVM analysis pass boilerplate
  static AnalysisKey Key;
  friend struct AnalysisInfoMixin<InfoPass>;

  using Loc = std::pair<int, int>;
  std::vector<Loc> parse_loop_locs();
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
