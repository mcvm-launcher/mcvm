#pragma once

// Formatting

#define _ESC "\033["
#define _FMT(code) _ESC << code

#define FMT_RESET _FMT("0m")
#define COL_RESET _FMT("39m")

#define BOLD_START _FMT("1m")
#define BOLD_END _FMT("22m")
#define BOLD(txt) BOLD_START << txt << BOLD_END

#define ITALIC_START _FMT("3m")
#define ITALIC_END _FMT("23m")
#define ITALIC(txt) ITALIC_START << txt << ITALIC_END

// Colors

#define RED_START _FMT("31m")
#define RED(txt) RED_START << txt << COL_RESET

#define YELLOW_START _FMT("33m")
#define YELLOW(txt) YELLOW_START << txt << COL_RESET

#define GRAY_START _FMT("90m")
#define GRAY(txt) GRAY_START << txt << COL_RESET

// Print value to cout
#define OUT(val) std::cout << val << '\n'
// Faster OUT for literal values
#define OUT_LIT(val) std::cout << (val "\n")
// OUT that replaces on a single line
#define OUT_REPL(val) std::cout << val << '\r' << std::flush
// Print a single newline
#define OUT_NEWLINE() std::cout << '\n'
// Print value to cerr
#define ERR(val) std::cerr << BOLD(RED_START << val) << '\n'

// Print value to cout only on debug builds
#if defined(NDEBUG)
	#define LOG(val) (void)0
#else
	#define LOG(val) std::cout << val << std::endl
#endif
