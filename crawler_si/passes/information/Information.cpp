#include "Information.h"

#include "llvm/Analysis/IVDescriptors.h"
#include "llvm/Analysis/LoopInfo.h"
#include "llvm/Analysis/ScalarEvolution.h"
#include "llvm/Analysis/ScalarEvolution.h"
#include "llvm/Analysis/ScalarEvolutionExpressions.h"
#include "llvm/IR/Constant.h"
#include "llvm/IR/Constants.h"
#include "llvm/IR/DebugLoc.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/Instruction.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/PassManager.h"
#include "llvm/Passes/OptimizationLevel.h"
#include "llvm/Passes/PassBuilder.h"
#include "llvm/Passes/PassPlugin.h"
#include "llvm/Support/Casting.h"
#include "llvm/Transforms/Scalar/LoopRotation.h"
#include "llvm/Transforms/Utils/LoopSimplify.h"

#include <algorithm>
#include <cstddef>
#include <cstring>
#include <string>
#include <unordered_set>
#include <vector>

using namespace std;
using namespace llvm;

namespace Info {

string format_str(string label, string value, bool last) {
  string acc;
  acc += label;
  acc += ": ";
  acc += value;
  if (!last) {
    acc += ", ";
  }

  return acc;
}

string InfoData::to_string() {
  string acc;

  acc += "loop info: ";

  acc += "[";
  acc += std::to_string(*this->locations.begin());
  std::for_each(
    std::next(this->locations.begin()), this->locations.end(),
    [&](const int& line){
      acc += " ";
      acc += std::to_string(line);
    });
  acc += "]";

  acc += " (";
  acc += format_str("ir_count", std::to_string(this->mix.count),       false);
  acc += format_str("ir_mem",   std::to_string(this->mix.mem_count),   false);
  acc += format_str("ir_arith", std::to_string(this->mix.arith_count), false);
  acc += format_str("ir_other", std::to_string(this->mix.other_count), false);


  string start = "null";
  if (this->pattern.start.has_value())
    start = std::to_string(this->pattern.start.value());

  string step = "null";
  if (this->pattern.step.has_value())
    step = std::to_string(this->pattern.step.value());

  acc += format_str("pat_start", start, false);
  acc += format_str("pat_step", step, true);
  acc += ")";

  return acc;
}

// =============================================================================
// Loop Finder Pass:
// =============================================================================

AnalysisKey InfoPass::Key;

InfoPass::Result InfoPass::run(Function &F, FunctionAnalysisManager &FAM) {
  Result data;

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

    // Has debug location info -> Has not been optimized away
    DebugLoc loc = BB.getFirstNonPHIOrDbg()->getDebugLoc();
    if (loc.get() == nullptr)
      continue;

    // Compute statistics
    auto locs = this->collect_locations(loop);
    IRMix mix = this->find_ir_mix(loop);
    MemPattern mem = this->find_mem_pattern(loop, FAM);
    data.push_back(InfoData(locs, mix, mem));
  }

  return data;
}

set<int> InfoPass::collect_locations(Loop *loop) {
  set<int> acc;

  for (auto &bb : loop->getBlocks()) {
    for (auto &inst : *bb) {
      DebugLoc loc = inst.getDebugLoc();
      if (loc) {
        int line = loc->getLine();
        if (line != 0)
          acc.insert(loc->getLine());
      }
    }
  }

  return acc;
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

MemPattern InfoPass::find_mem_pattern(Loop *loop, FunctionAnalysisManager &FAM) {
  MemPattern pattern;
  Function *fn = loop->getHeader()->getFirstNonPHI()->getFunction();
  ScalarEvolution &se = FAM.getResult<ScalarEvolutionAnalysis>(*fn);

  // Find the induction variable
  PHINode *iv = loop->getInductionVariable(se);
  if (iv == nullptr) {
    errs() << "Failed to find induction variable\n";
    return pattern;
  }

  InductionDescriptor desc;
  if (InductionDescriptor::isInductionPHI(iv, loop, &se, desc)) {
    // Get the IV start value
    ConstantInt *start = dyn_cast<ConstantInt>(desc.getStartValue());
    if (start != nullptr) {
      pattern.start = start->getSExtValue();
    }

    // Get the IV step
    const SCEVConstant *step = dyn_cast<SCEVConstant>(desc.getStep());
    if (step != nullptr) {
      pattern.step = step->getValue()->getSExtValue();
    }
  }

  return pattern;
}

// =============================================================================
// Loop Printer Pass:
// =============================================================================

PreservedAnalyses InfoPassPrinter::run(Function &F, FunctionAnalysisManager &FAM) {
  // Get the results of the loop finder pass
  auto data = FAM.getResult<InfoPass>(F);

  // Print out the matched locations
  for (auto &info : data) {
    errs() << info.to_string() << "\n";
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
            PB.registerVectorizerStartEPCallback(
              [](FunctionPassManager &FPM, OptimizationLevel Opt) {
                FPM.addPass(Info::InfoPassPrinter());
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
