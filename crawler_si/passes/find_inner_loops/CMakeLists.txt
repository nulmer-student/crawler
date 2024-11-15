include_directories(${CMAKE_CURRENT_BINARY_DIR} ${CMAKE_CURRENT_SOURCE_DIR})

file(GLOB source_files
  ${CMAKE_CURRENT_SOURCE_DIR}/FindInnerLoops.cpp
)

add_library(FindInnerLoops MODULE ${source_files})
target_link_libraries(FindInnerLoops LLVMSupport)
