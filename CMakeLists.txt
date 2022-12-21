cmake_minimum_required(VERSION 3.18)
project(mcvm_proj)

if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Debug)
endif()

set(CMAKE_CXX_FLAGS "-Wall -Wextra")
set(CMAKE_CXX_FLAGS_DEBUG "-g")
set(CMAKE_CXX_FLAGS_RELEASE "-O2")

set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 20)

find_package(RapidJSON)

add_executable(mcvm "")
target_link_libraries(mcvm curl)
# Required otherwise stuff segfaults
target_link_libraries(mcvm stdc++fs)
add_subdirectory(src)
