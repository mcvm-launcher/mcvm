#pragma once

#include <string>

// The current os as a string
#if defined(__linux__)
	#define OS_STRING "linux"
#elif defined(_WIN32)
	#define OS_STRING "windows"
#elif defined(__APPLE__)
	#define OS_STRING "osx"
#else
	#define OS_STRING ""
#endif

// The current arch as a string
#if defined(__x86_64)
	#define ARCH_STRING "x64"
#else
	#define ARCH_STRING ""
#endif

namespace mcvm {
	// For checking rule actions in Mojang json files
	static inline bool is_allowed(const std::string& action) {
		return (action == "allow");
	}
};
