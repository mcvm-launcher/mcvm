#pragma once
#include <iostream>

#define OUT(val) std::cout << val << "\n"
#define OUT_LIT(val) std::cout << (val "\n")
#define ERR(val) std::cerr << val << "\n"
#define ERR_LIT(val) std::cerr << (val "\n")

#define LOG(val)
#ifndef NDEBUG
	#define LOG(val) std::cout << val << std::endl
#endif
// Return if a condition is false
#define ENSURE(condition) if (condition) return;

namespace mcvm {

};
