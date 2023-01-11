#pragma once
#include <sys/stat.h>
#include <cstdlib>
#include <filesystem>
#include <string>
#include <cstring>
#include <iostream>
#include <exception>

namespace fs = std::filesystem;

// Relative paths to locations of mcvm files from mcvm base dir

#define ASSETS_INDEXES_DIR "indexes"
#define ASSETS_OBJECTS_DIR "objects"
#define ASSETS_VIRTUAL_DIR "virtual"

#define PROFILES_DIR "profiles"
#define INSTANCES_DIR "instances"
#define CLIENT_INSTANCES_DIR "client"
#define SERVER_INSTANCES_DIR "server"
#define CACHED_PACKAGES_DIR "pkg"

namespace mcvm {
	// Creates a directory at the path if it does not exist already
	extern void create_dir_if_not_exists(const fs::path& path);

	struct GetDirectoryException : public std::exception {
		const std::string dir;
		GetDirectoryException(const std::string& _dir) : dir(_dir) {}

		const char* what() {
			return ("Directory [" + dir + "] could not be located").c_str();
		}
	};

	// FIXME: On Windows I'm pretty sure these paths won't work
	// TODO: Add paths for MacOS

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

	static inline fs::path get_data_dir(const fs::path& home_dir = get_home_dir()) {
		#ifdef __linux__
			char* base_dir = std::getenv("XDG_DATA_HOME");
			if (base_dir) {
				return fs::path(base_dir) / "mcvm";
			}
			return home_dir / ".local" / "share" / "mcvm";
		#else
			#ifdef _WIN32
				return fs::path(std::getenv("APPDATA")) / "mcvm";
			#endif
		#endif
	}

	static inline fs::path get_internal_dir(const fs::path& mcvm_dir = get_data_dir()) {
		return mcvm_dir / "internal";
	}

	static inline fs::path get_cache_dir(const fs::path& home_dir = get_home_dir()) {
		#ifdef __linux__
			char* base_dir = std::getenv("XDG_CACHE_HOME");
			if (base_dir) {
				return fs::path(base_dir) / "mcvm";
			}
			return home_dir / ".cache" / "mcvm";
		#else
			#ifdef _WIN32
				return get_data_dir() / "cache";
			#endif
		#endif
	}

	static inline fs::path get_config_dir(const fs::path& home_dir = get_home_dir()) {
		#ifdef __linux__
			char* base_dir = std::getenv("XDG_CONFIG_HOME");
			if (base_dir) {
				return fs::path(base_dir) / "mcvm";
			}
			return home_dir / ".config" / "mcvm";
		#else
			#ifdef _WIN32
				return get_data_dir() / "config" / "mcvm";
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

	// Struct that holds all cached paths and is passed down through functions
	struct CachedPaths {
		const fs::path home;
		const fs::path data;
		const fs::path internal;
		const fs::path cache;
		const fs::path config;
		const fs::path run;
		const fs::path assets;

		CachedPaths()
		: home(get_home_dir()),
		data(get_data_dir(home)),
		internal(get_internal_dir(data)),
		cache(get_cache_dir(home)),
		config(get_config_dir(home)),
		run(get_run_dir()),
		assets(internal / "assets") {
			create_dir_if_not_exists(data);
			create_dir_if_not_exists(internal);
			create_dir_if_not_exists(cache);
			create_dir_if_not_exists(config);
		}
	};

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
