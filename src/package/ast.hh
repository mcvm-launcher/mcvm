#pragma once
#include "eval.hh"

#include <vector>
#include <string>

namespace mcvm {
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
};
