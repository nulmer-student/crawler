#include "FindInnerLoops.h"

#include <llvm/Analysis/LoopInfo.h>
#include <llvm/IR/Module.h>
#include <vector>

#include "llvm/IR/DebugLoc.h"
#include "llvm/IR/Instruction.h"
#include "llvm/IR/PassManager.h"
#include "llvm/Passes/PassPlugin.h"
#include "llvm/Passes/PassBuilder.h"

using namespace llvm;
using namespace InnerLoop;

// =============================================================================
// Loop Finder Pass:
// =============================================================================

AnalysisKey InnerLoopPass::Key;

InnerLoopPass::Result InnerLoopPass::run(Module &M, ModuleAnalysisManager &MAM) {
  Result Loops;

  // Get a function analysis manager
  FunctionAnalysisManager &FAM =
      MAM.getResult<FunctionAnalysisManagerModuleProxy>(M).getManager();

  // Loop over each basic block
  for (auto &Fn : M) {
    for (auto &BB : Fn) {
      auto &LoopInfo = FAM.getResult<LoopAnalysis>(Fn);
      Loop *L = LoopInfo.getLoopFor(&BB);

      // Basic block is not in a loop
      if (L == nullptr)
        continue;

      // Basic block is not the loop header
      if (!LoopInfo.isLoopHeader(&BB))
        continue;

      // Loop is not an intermost loop
      if (!L->isInnermost())
        continue;

      // Add the location of the first instruction
      DebugLoc Loc = BB.getFirstNonPHI()->getDebugLoc();
      Loops.push_back(Loc);
    }
  }

  return Loops;
}

// =============================================================================
// Loop Printer Pass:
// =============================================================================

PreservedAnalyses InnerLoopPassPrinter::run(Module &M, ModuleAnalysisManager &MAM) {
  // Get the results of the loop finder pass
  auto Results = MAM.getResult<InnerLoopPass>(M);

  // Print out the matched locations
  for (auto &Loc : Results) {
    errs() << Loc.getLine() << " " << Loc.getCol() << "\n";
  }

  return PreservedAnalyses::all();
}

llvm::PassPluginLibraryInfo getInnerLoopPluginInfo() {
  return {LLVM_PLUGIN_API_VERSION, "InnerLoop", LLVM_VERSION_STRING,
          [](PassBuilder &PB) {
            PB.registerPipelineParsingCallback(
                [](StringRef Name, ModulePassManager &MPM,
                   ArrayRef<PassBuilder::PipelineElement>) {
                  if (Name == "print<inner-loop>") {
                    MPM.addPass(InnerLoopPassPrinter());
                    return true;
                  }
                  return false;
                });
            PB.registerAnalysisRegistrationCallback(
                [](ModuleAnalysisManager &MAM) {
                  MAM.registerPass([&] { return InnerLoopPass(); });
                });
          }};
}

extern "C" LLVM_ATTRIBUTE_WEAK ::llvm::PassPluginLibraryInfo
llvmGetPassPluginInfo() {
  return getInnerLoopPluginInfo();
}
