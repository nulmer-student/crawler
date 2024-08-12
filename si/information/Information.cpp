#include "Information.h"

#include <cstring>
#include <llvm/Analysis/LoopInfo.h>
#include <llvm/IR/Function.h>
#include <llvm/IR/Module.h>

#include "llvm/IR/DebugLoc.h"
#include "llvm/IR/PassManager.h"
#include "llvm/Passes/PassPlugin.h"
#include "llvm/Passes/PassBuilder.h"
#include "llvm/Support/CommandLine.h"

#include <string>
#include <vector>

using namespace llvm;

namespace Info {

// Command line args:

cl::opt<std::string> LoopLocs(
  "loop-locs",
  cl::value_desc("locations"),
  cl::desc("Location of loops"),
  cl::Required);

// =============================================================================
// Loop Finder Pass:
// =============================================================================

AnalysisKey InfoPass::Key;

std::vector<InfoPass::Loc> InfoPass::parse_loop_locs() {
  std::vector<Loc> acc;

  // Current line / column
  int count = 0;
  int line = 0;

  // Split by spaces
  const char *delim = " ";
  char *split = std::strtok(LoopLocs.data(), delim);
  while (split != NULL) {
    int n = std::stoi(split);

    if (count % 2 == 0) {
      line = n;
    } else {
      acc.push_back(std::pair(line, n));
    }

    count++;
    split = strtok(NULL, delim);
  }

  return acc;
}

InfoPass::Result InfoPass::run(Function &F, FunctionAnalysisManager &FAM) {
  Result Loops;

  // Get the locations of relevent loops from the commandline
  std::vector<Loc> loop_locs = this->parse_loop_locs();

  for (auto loc : loop_locs) {
    errs() << loc.first << " " << loc.second << "\n";
  }

  return Loops;
}

// =============================================================================
// Loop Printer Pass:
// =============================================================================

PreservedAnalyses InfoPassPrinter::run(Function &F, FunctionAnalysisManager &FAM) {
  // Get the results of the loop finder pass
  auto Results = FAM.getResult<InfoPass>(F);

  // Print out the matched locations
  for (auto &Loc : Results) {
    errs() << Loc.getLine() << " " << Loc.getCol() << "\n";
  }

  return PreservedAnalyses::all();
}

}

llvm::PassPluginLibraryInfo getInnerLoopPluginInfo() {
  return {LLVM_PLUGIN_API_VERSION, "Info", LLVM_VERSION_STRING,
          [](PassBuilder &PB) {
            PB.registerPipelineParsingCallback(
                [](StringRef Name, FunctionPassManager &FPM,
                   ArrayRef<PassBuilder::PipelineElement>) {
                  if (Name == "print<info>") {
                    FPM.addPass(Info::InfoPassPrinter());
                    return true;
                  }
                  return false;
                });
            PB.registerAnalysisRegistrationCallback(
                [](FunctionAnalysisManager &FAM) {
                  FAM.registerPass([&] { return Info::InfoPass(); });
                });
          }};
}

extern "C" LLVM_ATTRIBUTE_WEAK ::llvm::PassPluginLibraryInfo
llvmGetPassPluginInfo() {
  return getInnerLoopPluginInfo();
}
