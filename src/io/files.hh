#pragma once
#include "paths.hh"

namespace mcvm {
	// Returns whether a file at the specified path exists
	extern bool file_exists(const std::string& path);

	// Creates a directory at the path if it does not exist already
	extern void create_dir_if_not_exists(const fs::path& path);

	// Returns the length of a path by iterating over it
	extern std::size_t path_length(const fs::path& path);

	// Creates the directories leading up to a file path if they do not already exist
	extern void create_leading_directories(const fs::path& path);
};
