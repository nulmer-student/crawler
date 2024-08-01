#include "FindInnerLoops.h"

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
  return Loops;
}

// =============================================================================
// Loop Printer Pass:
// =============================================================================

PreservedAnalyses InnerLoopPassPrinter::run(Module &M, ModuleAnalysisManager &MAM) {
  // Get the results of the loop finder pass
  auto Results = MAM.getResult<InnerLoopPass>(M);

  errs() << "this is a test\n";

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
