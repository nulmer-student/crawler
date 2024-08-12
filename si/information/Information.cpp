#include "Information.h"

#include <algorithm>
#include <cstring>
#include <llvm/Analysis/LoopInfo.h>
#include <llvm/IR/Function.h>
#include <llvm/IR/Instructions.h>
#include <llvm/IR/Module.h>

#include "llvm/IR/DebugLoc.h"
#include "llvm/IR/PassManager.h"
#include "llvm/Passes/PassPlugin.h"
#include "llvm/Passes/PassBuilder.h"
#include "llvm/Support/CommandLine.h"

#include <llvm/Support/Casting.h>
#include <string>
#include <unordered_set>
#include <utility>
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

InfoPass::Locs InfoPass::parse_loop_locs() {
  Locs acc;
  int count = 0;

  // Copy the input
  std::string locs = LoopLocs;

  // Split by spaces
  const char *delim = " ";
  char *split = std::strtok(locs.data(), delim);
  while (split != NULL) {
    int n = std::stoi(split);

    // Only save the line numbers
    if (count % 2 == 0) {
      acc.insert(n);
    }

    count++;
    split = strtok(NULL, delim);
  }

  return acc;
}

InfoPass::Result InfoPass::run(Function &F, FunctionAnalysisManager &FAM) {
  Result Loops;

  // Get the locations of relevent loops from the commandline
  Locs loop_locs = this->parse_loop_locs();

  // Extract the information for each relevent loop in the IR
  for (auto &BB : F) {
    auto &loop_info = FAM.getResult<LoopAnalysis>(F);
    Loop *loop = loop_info.getLoopFor(&BB);

    // Basic block is not in a loop
    if (loop == nullptr)
      continue;

    // Basic block is not the loop header
    if (!loop_info.isLoopHeader(&BB))
      continue;

    // Has debug location info
    DebugLoc loc = BB.begin()->getDebugLoc();
    if (loc.get() == nullptr)
      continue;

    // The loop is not one of the relevent loops
    int line = loc.getLine();
    if (loop_locs.find(line) == loop_locs.end())
      continue;

    // Compute statistics
    IRMix mix = this->find_ir_mix(loop);
    errs() << mix.count
           << " " << mix.mem_count
           << " " << mix.arith_count
           << " " << mix.other_count
           << "\n";
  }

  return Loops;
}

// Opcode names of arithmetic instructions
const std::unordered_set<std::string> ARITH_INST = {
  "fneg", "add",  "fadd", "sub",  "fsub", "mul",  "fmul",
  "udiv", "sdiv", "fdiv", "urem", "srem", "frem", "shl",
  "lshr", "ashr", "and",  "or",   "xor"
};

IRMix InfoPass::find_ir_mix(Loop *loop) {
  IRMix counts;

  // Iterate over all instructions in the loop
  for (auto &bb : loop->getBlocks()) {
    for (auto &inst : *bb) {
      // Count the total number
      counts.count++;

      // Arithmetic instructions
      std::string name = inst.getOpcodeName();
      if (ARITH_INST.find(name) != ARITH_INST.end()) {
        counts.arith_count++;
        continue;
      }

      // Memory instructions
      if (isa<LoadInst>(inst) || isa<StoreInst>(inst)) {
        counts.mem_count++;
        continue;
      }

      // Otherwise, add to the default count
      counts.other_count++;
    }
  }

  return counts;
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
