#pragma once
#include "package.hh"

#include <map>

namespace mcvm {
	// A node in the abstract syntax tree
	class PkgNode {
		public:
		std::string text;
	};

	class PkgInstruction : public PkgNode {
		public:
		virtual void evaluate(PkgEvalResult& result, RunLevel level) {}

		virtual ~PkgInstruction() = default;
	};

	class PkgBlock {
		public:
		PkgBlock() = default;

		std::vector<PkgInstruction*> instructions;
		PkgBlock* parent = nullptr;

		void evaluate(PkgEvalResult& result, RunLevel level);
	};

	class PkgAST {
		public:
		std::map<std::string, PkgBlock> routines;

		PkgAST() = default;

		~PkgAST() {
			for (std::map<std::string, PkgBlock>::iterator i = routines.begin(); i != routines.end(); i++) {
				PkgBlock rtn = i->second;
				for (std::vector<PkgInstruction*>::iterator j = rtn.instructions.begin(); j != rtn.instructions.end(); j++) {
					delete *j;
					rtn.instructions.erase(j);
				}
				routines.erase(i);
			}
		}
	};
	
	// The different objects that are evaluated
	enum ParseType {
		ROOT, // A command or routine, at the start of an instruction
		STRING
	};

	struct ParseRoot {
		std::vector<std::string> words = { "" };
		bool is_routine = false;
	};

	struct ParseString {
		enum StringState {
			OUTSIDE,
			BEGIN,
			INSIDE,
			END
		};
		StringState state = OUTSIDE;
		bool multiline = false;
		unsigned short quote_count = 0; // Number of quotes in the string, used to determine single vs multiline
		std::string str;
	};

	struct ParseDebug {
		std::string evaluated_chars;
	};

	struct ParseData {
		PkgAST* ast;

		// Location
		
		unsigned int instruction = 0; // The instruction or line number
		unsigned int line = 0; // The line number
		unsigned int character = 0; // The character
		int char_in_instruction = 0; // The character relative to this instruction

		// Context

		RunLevel current_run_level = RunLevel::NONE; // Run level that is used with respect to routine
		RunLevel user_run_level; // Run level that the user set
		std::string routine;
		PkgBlock* current_block = nullptr;
		PkgBlock* default_routine_block = nullptr;
		ParseType expected_type = ParseType::ROOT; // Expected type of next character

		// Extra

		char last_char;
		
		// Subsections

		ParseString string;
		ParseRoot root;
	#ifndef NDEBUG
		ParseDebug debug;
	#endif
	};

	// Exceptions

	struct PkgSyntaxError : public std::exception {
		std::string msg;
		unsigned int row;
		unsigned int col;

		PkgSyntaxError(std::string _msg, unsigned int _row, unsigned int _col)
		: msg(_msg), row(_row), col(_col) {}
	};
}
