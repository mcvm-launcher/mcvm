#pragma once
#include <sys/stat.h>
#include <cstdlib>
#include <filesystem>
#include <string>
#include <cstring>
#include <iostream>

namespace fs = std::filesystem;

// Path creation utilities
#define PATH_SEP "/"
#define PATH_CONCAT_2(path1, path2) path1 PATH_SEP path2
#define PATH_CONCAT_3(path1, path2, path3) PATH_CONCAT_2(PATH_CONCAT_2(path1, path2), path3)
#define PATH_CONCAT_4(path1, path2, path3, path4) PATH_CONCAT_2(PATH_CONCAT_3(path1, path2, path3), path4)
#define PATH_CONCAT_5(path1, path2, path3, path4, path5) PATH_CONCAT_2(PATH_CONCAT_4(path1, path2, path3, path4), path5)
#define PATH_CONCAT_6(path1, path2, path3, path4, path5, path6) PATH_CONCAT_2(PATH_CONCAT_5(path1, path2, path3, path4, path5), path6)

// Relative paths to locations of mcvm files from mcvm base dir
#define ASSETS_DIR "assets"

namespace mcvm {
	static fs::path get_home_dir() {
		#ifdef __linux__
			return fs::path(std::getenv("HOME"));
		#else
			#ifdef _WIN32
				return fs::path("C:")
			#endif
		#endif
	}

	static fs::path get_mcvm_dir() {
		#ifdef __linux__
			return get_home_dir() / fs::path(".local" PATH_SEP "share" PATH_SEP "mcvm");
		#else
			#ifdef _WIN32
				return get_home_dir() / fs::path("mcvm");
			#endif
		#endif
	}
};
