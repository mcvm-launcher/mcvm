#pragma once
#include "eval.hh"

#include <vector>
#include <string>

namespace mcvm {
	// A node in the abstract syntax tree
	class PkgNode {
		public:
		std::string text;
	};

	class PkgInstruction : public PkgNode {
		public:
		virtual void evaluate(PkgEvalResult& result, ParseData& prs) {}

		virtual ~PkgInstruction() = default;
	};

	struct PkgIfCondition {
		enum Condition {
			MATCH,
			VERSION,
			MODLOADER
		};
		Condition condition;
		std::string left_side;
		std::string right_side;
	};

	class PkgIfInstruction : public PkgInstruction {
		public:
		PkgBlock* nested_block = nullptr;
		PkgIfCondition condition;

		void evaluate(PkgEvalResult& result, ParseData& prs) override {

		}

		~PkgIfInstruction() {
			assert(nested_block != nullptr);
			delete nested_block;
		}
	};
	
	class PkgCommandInstruction : public PkgInstruction {
		public:
		std::string command;
		std::vector<std::string> args;
	};
};
