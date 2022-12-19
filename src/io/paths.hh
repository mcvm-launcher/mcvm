#pragma once
#include <sys/stat.h>
#include <cstdlib>
#include <string>
#include <cstring>

// Path creation utilities
#define PATH_SEP "/"
#define PATH_CONCAT_2(path1, path2) path1 PATH_SEP path2
#define PATH_CONCAT_3(path1, path2, path3) PATH_CONCAT_2(PATH_CONCAT_2(path1, path2), path3)
#define PATH_CONCAT_4(path1, path2, path3, path4) PATH_CONCAT_2(PATH_CONCAT_3(path1, path2, path3), path4)
#define PATH_CONCAT_5(path1, path2, path3, path4, path5) PATH_CONCAT_2(PATH_CONCAT_4(path1, path2, path3, path4), path5)
#define PATH_CONCAT_6(path1, path2, path3, path4, path5, path6) PATH_CONCAT_2(PATH_CONCAT_5(path1, path2, path3, path4, path5), path6)

// Base path definitions
#define MCVM_DIR path_concat(HOME_DIR, PATH_CONCAT_3(".local", "share", "mcvm"))
#ifdef WIN32
	// TODO: Actual path with user detection, in appdata or something
	#define MCVM_DIR PATH_CONCAT_2("C:", "mcvm")
#endif

// Relative paths to locations of mcvm files from mcvm base dir
#define ASSETS_DIR "assets"

namespace mcvm {
	static const std::string path_concat(const std::string& str1, const std::string& str2) {
		return str1 + PATH_SEP + str2;
	}

	static std::string get_home_dir() {
		return std::getenv("HOME");
	}

	static std::string get_mcvm_dir() {
		return path_concat(get_home_dir(), std::string(".local" PATH_SEP "share" PATH_SEP "mcvm"));
	}

	static const bool file_exists(const std::string& path) {
		struct stat buffer;
		return (stat (path.c_str(), &buffer) == 0); 
	}
};
