add_subdirectory(java)
add_subdirectory(files)

target_sources(mcvm PUBLIC
	game.cc
	config.cc
)
