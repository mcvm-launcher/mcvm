#pragma once
#include <iostream>

// Print value to cout
#define OUT(val) std::cout << val << "\n"
// Faster OUT for literal values
#define OUT_LIT(val) std::cout << (val "\n")
// Print value to cerr
#define ERR(val) std::cerr << val << "\n"
// Faster ERR for literal values
#define ERR_LIT(val) std::cerr << (val "\n")

// Print value to cout only on debug builds
#define LOG(val)
#ifndef NDEBUG
	#define LOG(val) std::cout << val << std::endl
#endif

// Return if a condition is true
#define GUARD(condition) if (condition) return
// Return if a condition is false
#define ENSURE(condition) GUARD(!(condition))

// The current os as a string
#define OS_STRING ""
#ifdef __linux__
	#define OS_STRING "linux"
#endif
#ifdef _WIN32
	#define OS_STRING "windows"
#endif
#ifdef __APPLE__
	#define OS_STRING "osx"
#endif

namespace mcvm {
	// Compute the length of a string literal at compile time\
	// https://stackoverflow.com/a/26082447
	template <std::size_t N>
	constexpr std::size_t litlen(const char[N]) {
		return N - 1;
	}
};
