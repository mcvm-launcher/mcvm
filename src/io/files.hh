#pragma once
#include "paths.hh"

namespace mcvm {
	extern bool file_exists(const std::string& path);

	extern void create_dir_if_not_exists(const fs::path& path);

	extern std::size_t path_length(const fs::path& path);

	// Creates the directories leading up to a file path if they do not already exist
	extern void create_leading_directories(const fs::path& path);
};
