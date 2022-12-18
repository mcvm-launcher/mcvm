cmake_minimum_required(VERSION 3.18)
project(mcvm_proj)

set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 20)

add_executable(mcvm "")
add_subdirectory(src)
