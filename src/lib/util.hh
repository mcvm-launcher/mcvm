#pragma once

#include <rapidjson/document.h>

#include <iostream>
#include <vector>

// Output

// Print value to cout
#define OUT(val) std::cout << val << '\n'
// Faster OUT for literal values
#define OUT_LIT(val) std::cout << (val "\n")
// OUT that replaces on a single line
#define OUT_REPL(val) std::cout << val << '\r' << std::flush
// Print a single newline
#define OUT_NEWLINE() std::cout << '\n'
// Print value to cerr
#define ERR(val) std::cerr << val << '\n'
// Faster ERR for literal values
#define ERR_LIT(val) std::cerr << (val '\n')

// Print value to cout only on debug builds
#define LOG(val)
#ifndef NDEBUG
	#define LOG(val) std::cout << val << std::endl
#endif

// Error checking

// Return if a condition is true
#define GUARD(condition) if (condition) return
// Return if a condition is false
#define ENSURE(condition) GUARD(!(condition))
// Assert that this statement will never be run
#define ASSERT_NOREACH() assert(false)

// Memory utilities

// Delete pointed to elements of a vector but do not delete the elements themselves
#define DEL_VECTOR(vec) for (unsigned int __i = 0; __i < vec.size(); __i++) { delete vec[__i]; }
// Delete an object with a nullptr check
#define PROTECTED_DEL(obj) (assert(obj != nullptr); delete obj)

// The current os as a string
#ifdef __linux__
	#define OS_STRING "linux"
#endif
#ifdef _WIN32
	#define OS_STRING "windows"
#endif
#ifdef __APPLE__
	#define OS_STRING "osx"
#endif

// Attributes

#define UNUSED [[maybe_unused]]
#define FALLTHROUGH [[fallthrough]]

namespace json = rapidjson;

namespace mcvm {
	// Compute the length of a string literal at compile time
	// https://stackoverflow.com/a/26082447
	template <std::size_t N>
	constexpr std::size_t litlen(const char[N]) {
		return N - 1;
	}
	// Finds and replaces first occurrence of a string in another string and replaces it with something else
	// Will modify source
	static inline void fandr(std::string& source, const std::string& find, const std::string_view repl) {
		std::size_t pos = source.find(find);
		if (pos == std::string::npos) return;
		source.replace(pos, find.length(), repl);
	}

	// Access a json value with an assertion that it is there
	static inline json::Value& json_access(json::Value& val, const char* key) {
		assert(val.HasMember(key));
		return val[key];
	}

	static inline json::Value& json_access(json::Value* val, const char* key) {
		assert(val->HasMember(key));
		return val->operator[](key);
	}

	static inline json::Value& json_access(json::GenericObject<false, json::Value>& val, const char* key) {
		assert(val.HasMember(key));
		return val[key];
	}

	// Obtain a subvector of a parent vector
	template <typename T>
	static inline std::vector<T> vec_slice(const std::vector<T>& src, std::size_t start, std::size_t len) {
		auto first = src.cbegin() + start;
		auto second = first + len;
		return std::vector<T>(first, second);
	}
};
