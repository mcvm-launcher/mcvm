#include "package.hh"

namespace mcvm {
	bool mod_supported(const PkgEvalGlobals& global, const ModType& loader) {
		// TODO: Mod bridges
		switch (loader) {
			case ModType::FORGE:
			case ModType::QUILT:
				return (global.modloader == loader);
			case ModType::FABRIC:
				return (global.modloader == ModType::FABRIC || global.modloader == ModType::QUILT);
			default:
				return false;
		}
	}

	void PkgBlock::evaluate(PkgEvalData& data, const PkgEvalGlobals& global) {
		for (uint i = 0; i < instructions.size(); i++) {
			instructions[i]->evaluate(data, global);
		}
	}

	void PkgCommandInstruction::evaluate(PkgEvalData& data, const PkgEvalGlobals& global) {
		std::cout << text;
		for (uint i = 0; i < args.size(); i++) {
			std::cout << ' ';
			std::cout << args[i];
		}
		OUT_NEWLINE();

		switch (command) {
			case PkgCommandInstruction::SET_NAME:
				data.pkg_name = args.at(1);
				break;
			case PkgCommandInstruction::SET_VERSION:
				data.pkg_version = args.at(1);
				break;
			case PkgCommandInstruction::RESOURCE_TYPE:
				break;
		}
	}

	void PkgIfInstruction::evaluate(PkgEvalData& data, const PkgEvalGlobals& global) {
		GUARD(global.level == RunLevel::NONE);

		bool condition_success = false;
		switch (condition.condition) {
			case PkgIfCondition::MATCH:
				condition_success = (condition.left_side == condition.right_side);
				break;
			case PkgIfCondition::VERSION:
				condition_success = (condition.left_side == global.mc_version);
				break;
			case PkgIfCondition::MODLOADER: {
				static const std::map<std::string, ModType> mod_map = {
					{"forge", ModType::FORGE},
					{"fabric", ModType::FABRIC},
					{"quilt", ModType::QUILT}
				};
				condition_success = mod_supported(global, mod_map.at(condition.left_side));
				break;
			}
			case PkgIfCondition::SIDE: {
				static const std::map<std::string, MinecraftSide> side_map = {
					{"client", MinecraftSide::CLIENT},
					{"server", MinecraftSide::SERVER}
				};
				condition_success = (side_map.at(condition.left_side) == global.side);
				break;
			}
		}
		if (condition.inverted) condition_success = !condition_success;

		OUT_LIT("if {");
		if (condition_success) {
			nested_block.evaluate(data, global);
		}
		OUT_LIT("}");
	}

	void Package::evaluate(PkgEvalData& data, const std::string& routine_name, const PkgEvalGlobals& global) {
		PkgBlock& routine = ast->routines.at(routine_name);
		routine.evaluate(data, global);
	}
};
