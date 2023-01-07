#include "ast.hh"

namespace mcvm {
	bool mod_supported(PkgEvalResult& result, const ModType& loader) {
		// TODO: Mod bridges
		switch (loader) {
			case ModType::FORGE:
			case ModType::QUILT:
				return (result.modloader == loader);
			case ModType::FABRIC:
				return (result.modloader == ModType::FABRIC || result.modloader == ModType::QUILT);
		}
	}

	void PkgBlock::evaluate(PkgEvalResult& result, RunLevel level) {
		for (unsigned int i = 0; i < instructions.size(); i++) {
			instructions[i]->evaluate(result, level);
		}
	}

	void PkgCommandInstruction::evaluate(PkgEvalResult& result, RunLevel level) {
		std::cout << text;
		for (unsigned int i = 0; i < args.size(); i++) {
			std::cout << ' ';
			std::cout << args[i];
		}
		OUT_NEWLINE();
	}

	void PkgIfInstruction::evaluate(PkgEvalResult& result, RunLevel level) {
		bool condition_success = false;
		switch (condition.condition) {
			case PkgIfCondition::MATCH:
				condition_success = (condition.left_side == condition.right_side);
				break;
			case PkgIfCondition::VERSION:
				condition_success = (condition.left_side == result.mc_version);
				break;
			case PkgIfCondition::MODLOADER: {
				const std::map<std::string, ModType> mod_map = {
					{"forge", ModType::FORGE},
					{"fabric", ModType::FABRIC},
					{"quilt", ModType::QUILT}
				};
				condition_success = mod_supported(result, mod_map.at(condition.left_side));
				break;
			}
			case PkgIfCondition::SIDE: {
				const std::map<std::string, MinecraftSide> side_map = {
					{"client", MinecraftSide::CLIENT},
					{"server", MinecraftSide::SERVER}
				};
				condition_success = (side_map.at(condition.left_side) == result.side);
				break;
			}
		}
		if (condition.inverted) condition_success = !condition_success;

		// TODO: temporary
		OUT_LIT("if {");
		if (condition_success) {
			nested_block->evaluate(result, level);
		}
		OUT_LIT("}");
	}

	void Package::evaluate(PkgEvalResult& ret, const std::string& routine_name, RunLevel level) {
		ret.pkg_name = name;
		PkgBlock& routine = ast->routines.at(routine_name);
		routine.evaluate(ret, level);
	}
};
