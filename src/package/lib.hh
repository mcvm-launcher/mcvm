#pragma once
#include "lib/util.hh"

#include <map>

namespace mcvm {
	struct PkgEvalData;
	struct PkgEvalGlobals;

	// A node in the abstract syntax tree
	class PkgNode {
		public:
		std::string text;
	};

	class PkgInstruction : public PkgNode {
		public:
		virtual void evaluate(UNUSED PkgEvalData& data, UNUSED const PkgEvalGlobals& global) {}

		virtual ~PkgInstruction() = default;
	};

	class PkgBlock {
		public:
		PkgBlock() = default;

		std::vector<PkgInstruction*> instructions;
		PkgBlock* parent = nullptr;

		void evaluate(PkgEvalData& data, const PkgEvalGlobals& global);
	};

	// Used to download resources at the end
	class ResourceAquirer {
		public:
		ResourceAquirer();
	};

	// The level of evaluation to be performed
	enum RunLevel {
		ALL, // Run all commands
		RESTRICTED, // Restrict the scope of commands
		INFO, // Only run commands that set information
		NONE // Don't run any commands
	};
	
	// Package eval global information
	struct PkgEvalGlobals {
		RunLevel level = RunLevel::ALL;
		fs::path working_directory;
		std::string package_requested_version;
		MCVersionString mc_version;
		ModType modloader = ModType::FABRIC; 
		MinecraftSide side = MinecraftSide::CLIENT;
	};

	// The result from evaluation
	struct PkgEvalData {
		std::string pkg_name;
		std::string pkg_version;
		// TODO: Temporary
		// The list of resources to be aquired once the whole package is evaluated
		std::vector<ResourceAquirer*> resources;

		~PkgEvalData() {
			DEL_VECTOR(resources);
		}
	};

	class PkgAST {
		public:
		std::map<std::string, PkgBlock> routines;

		PkgAST() = default;

		~PkgAST() {
			for (std::map<std::string, PkgBlock>::iterator i = routines.begin(); i != routines.end(); i++) {
				PkgBlock rtn = i->second;
				DEL_VECTOR(rtn.instructions);
			}
		}
	};

	struct PkgIfCondition {
		enum Condition {
			NOT,
			MATCH,
			VERSION,
			MODLOADER,
			SIDE
		};
		Condition condition;
		std::string left_side;
		std::string right_side;
		bool inverted = false;
	};

	class PkgIfInstruction : public PkgInstruction {
		public:
		PkgBlock nested_block;
		PkgIfCondition condition;

		void evaluate(PkgEvalData& data, const PkgEvalGlobals& global) override;
	};
	
	class PkgCommandInstruction : public PkgInstruction {
		public:
		enum PkgCommand {
			SET_NAME,
			SET_VERSION,
			RESOURCE_TYPE,
			RESOURCE_NAME,
			DOWNLOAD_RESOURCE,
			FINISH,
			FAIL
		};

		PkgCommand command;
		std::vector<std::string> args;

		void evaluate(PkgEvalData& data, const PkgEvalGlobals& global) override;
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
};
