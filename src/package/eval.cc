#include "package.hh"

namespace mcvm {
	// The different objects that are evaluated
	enum ParseType {
		ROOT, // A command or routine, at the start of an instruction
		STRING
	};

	struct ParseCommand {
		std::string command;
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
		// Location
		
		unsigned int instruction = 0; // The instruction or line number
		unsigned int line = 0; // The line number
		unsigned int character = 0; // The character

		// Context

		RunLevel current_run_level = RunLevel::NONE; // Run level that is used with respect to routine
		RunLevel user_run_level; // Run level that the user set
		std::string routine;

		// Extra

		char last_char;
		
		// Subsections

		ParseString string;
	#ifndef NDEBUG
		ParseDebug debug;
	#endif
	};

	inline void eval_char(const char& c, ParseData& prs) {
		
	}

	void Package::evaluate(PkgEvalResult& ret, const std::string& routine, RunLevel level) {
		ParseData prs;
		prs.user_run_level = level;
		prs.routine = routine;
		
		for (unsigned int i = 0; i < contents.length(); i++) {
			prs.character = i;
			const char& c = contents[i];

			switch (c) {
				FALLTHROUGH case ';':
					if (prs.string.state != ParseString::INSIDE && prs.string.state != ParseString::BEGIN) {
						prs.instruction++;
						break;
					}
				case '\n':
					prs.line++;
					prs.instruction++;
					break;
				default:
					eval_char(c, prs);
			}

			#ifndef NDEBUG
				prs.debug.evaluated_chars.push_back(c);
			#endif
			prs.last_char = c;
		}
	}
};
