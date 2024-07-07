add_executable(miner
  Main.cpp
  Util.cpp
  Deps.cpp
  Include.cpp
  Compile.cpp
  PreProcess.cpp
)

target_link_libraries(miner PRIVATE OpenMP::OpenMP_CXX)
