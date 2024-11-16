include_directories(${CMAKE_CURRENT_BINARY_DIR} ${CMAKE_CURRENT_SOURCE_DIR})

file(GLOB source_files
  ${CMAKE_CURRENT_SOURCE_DIR}/Information.cpp
)

add_library(Information MODULE ${source_files})
target_link_libraries(Information LLVMSupport)
