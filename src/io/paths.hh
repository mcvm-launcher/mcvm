#pragma once
#include <sys/stat.h>
#include <cstdlib>
#include <filesystem>
#include <string>
#include <cstring>
#include <iostream>
#include <exception>

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
#define PROFILES_DIR "profiles"
#define INSTANCES_DIR "instances"
#define CLIENT_INSTANCES_DIR "client"
#define SERVER_INSTANCES_DIR "server"
#define CACHED_PACKAGES_DIR "pkg"

namespace mcvm {
	struct GetDirectoryException : public std::exception {
		const std::string dir;
		GetDirectoryException(const std::string& _dir) : dir(_dir) {}

		const char* what() {
			return ("Directory [" + dir + "] could not be located").c_str();
		}
	};

	static inline fs::path get_home_dir() {
		#ifdef __linux__
			char* home_dir = std::getenv("XDG_HOME");
			if (!home_dir) {
				home_dir = std::getenv("HOME");
			}
			if (!home_dir) {
				throw GetDirectoryException{"home"};
			}
			return fs::path(home_dir);
		#else
			#ifdef _WIN32
				return fs::path("C:")
			#endif
		#endif
	}

	static inline fs::path get_mcvm_dir(const fs::path& home_dir = get_home_dir()) {
		#ifdef __linux__
			char* base_dir = std::getenv("XDG_DATA_HOME");
			if (base_dir) {
				return fs::path(base_dir) / "mcvm";
			}
			return home_dir / fs::path(".local" PATH_SEP "share" PATH_SEP "mcvm");
		#else
			#ifdef _WIN32
				return fs::path(std::getenv("APPDATA")) / "mcvm";
			#endif
		#endif
	}

	static inline fs::path get_cache_dir(const fs::path& home_dir = get_home_dir()) {
		#ifdef __linux__
			char* base_dir = std::getenv("XDG_CACHE_HOME");
			if (base_dir) {
				return fs::path(base_dir) / "mcvm";
			}
			return home_dir / fs::path(".cache" PATH_SEP "mcvm");
		#else
			#ifdef _WIN32
				return get_mcvm_dir() / "cache";
			#endif
		#endif
	}

	static inline fs::path get_run_dir() {
		#ifdef __linux__
			char* base_dir = std::getenv("XDG_RUNTIME_DIR");
			if (base_dir) {
				return fs::path(base_dir);
			}
			return fs::path("/run/user") / std::getenv("UID");
		#else
			#ifdef _WIN32
				return get_cache_dir();
			#endif
		#endif
	}

	// File extensions
	static inline std::string add_package_extension(const std::string& name) {
		return name + ".pkg.txt";
	}

	struct FileOpenError : public std::exception {
		FileOpenError(const char* _filename) : filename(_filename) {} 
		const char* filename;
		const char* what() {
			return (std::string() + "File " + filename + " could not be opened").c_str();
		}
	};
};
