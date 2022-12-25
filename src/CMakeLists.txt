target_include_directories(mcvm PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})

add_subdirectory(lib)
add_subdirectory(data)
add_subdirectory(package)
add_subdirectory(user)
add_subdirectory(io)
add_subdirectory(net)
add_subdirectory(commands)

target_sources(mcvm PUBLIC
	main.cc
	daemon.cc
)
