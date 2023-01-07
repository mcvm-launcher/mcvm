#pragma once
#include "paths.hh"

#include <fstream>

namespace mcvm {
	// Returns whether a file at the specified path exists
	extern bool file_exists(const std::string& path);

	// Returns the length of a path by iterating over it
	extern std::size_t path_length(const fs::path& path);

	// Creates the directories leading up to a file path if they do not already exist
	extern void create_leading_directories(const fs::path& path);

	// Reads from a file into a string using an ifstream
	extern void read_file(const fs::path& path, std::string& out);

	// Writes chars to a file using a file handler
	extern void write_file(const fs::path& path, const char* text);
};
