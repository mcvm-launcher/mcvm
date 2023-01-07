#include "ast.hh"

namespace mcvm {
	void reset_instruction(ParseData& prs) {
		prs.instruction++;
		prs.char_in_instruction = -1;
		prs.root.words = { "" };
		prs.root.is_routine = false;
	}

	void new_instruction(ParseData& prs) {
		const std::string& instruction = prs.root.words.front();
		ENSURE(instruction != "");

		if (prs.root.is_routine) {
			PkgBlock routine{};
			auto pair = prs.ast->routines.insert(std::make_pair(instruction, routine));
			prs.current_block = &pair.first->second;
		} else {
			// Parse specific instructions
			// TODO: Errors / arg checking
			if (instruction == "if") {
				PkgIfInstruction* inst = new PkgIfInstruction;

				PkgIfCondition cond;
				const std::map<std::string, PkgIfCondition::Condition> cond_map = {
					{"not", PkgIfCondition::NOT},
					{"match", PkgIfCondition::MATCH},
					{"version", PkgIfCondition::VERSION},
					{"modloader", PkgIfCondition::MODLOADER},
					{"side", PkgIfCondition::SIDE}
				};
				unsigned int arg_root_pos = 1;
				cond.condition = cond_map.at(prs.root.words.at(arg_root_pos));
				if (cond.condition == PkgIfCondition::NOT) {
					cond.inverted = true;
					arg_root_pos++;
					cond.condition = cond_map.at(prs.root.words.at(arg_root_pos));
				}
				cond.left_side = prs.root.words.at(arg_root_pos + 1);
				if (prs.root.words.size() > arg_root_pos + 2) {
					cond.right_side = prs.root.words.at(arg_root_pos + 2);
				}
				inst->condition = cond;

				inst->nested_block = new PkgBlock;
				inst->nested_block->parent = prs.current_block;
				prs.current_block->instructions.push_back(inst);
				prs.current_block = inst->nested_block;
			} else {
				PkgCommandInstruction* inst = new PkgCommandInstruction;
				const std::map<std::string, PkgCommandInstruction::PkgCommand> command_map = {
					{"name", PkgCommandInstruction::SET_NAME},
					{"version", PkgCommandInstruction::SET_VERSION},
					{"resource-type", PkgCommandInstruction::RESOURCE_TYPE},
					{"resource-name", PkgCommandInstruction::RESOURCE_NAME},
					{"download-resource", PkgCommandInstruction::DOWNLOAD_RESOURCE},
					{"finish", PkgCommandInstruction::FINISH},
					{"fail", PkgCommandInstruction::FAIL}
				};
				inst->command = command_map.at(instruction);
				inst->text = instruction;
				inst->args = vec_slice(prs.root.words, 1, prs.root.words.size() - 1);
				prs.current_block->instructions.push_back(inst);
			}
		}
		reset_instruction(prs);
	}

	void parse_root(const char& c, ParseData& prs) {
		switch (c) {
			case ' ':
				prs.root.words.push_back("");
				break;
			case '@':
				if (prs.char_in_instruction == 0) {
					prs.root.is_routine = true;
				}
				FALLTHROUGH;
			default:
				prs.root.words.back() += c;
		}
	}

	inline void eval_char(const char& c, ParseData& prs) {
		switch (prs.expected_type) {
			case ParseType::ROOT:
				parse_root(c, prs);
				break;
		}
	}

	void Package::parse() {
		ParseData prs;
		ast = new PkgAST;
		prs.ast = ast;
		auto pair = ast->routines.insert(std::make_pair("__default", PkgBlock{}));
		PkgBlock* block = &pair.first->second;
		prs.default_routine_block = prs.current_block = block;
		
		for (unsigned int i = 0; i < contents.length(); i++) {
			prs.character = i;
			const char& c = contents[i];

			switch (c) {
				case '{':
				case ';':
					if (prs.string.state != ParseString::INSIDE && prs.string.state != ParseString::BEGIN) {
						new_instruction(prs);
						break;
					}
					FALLTHROUGH;
				case '}':
					if (prs.char_in_instruction == 0) {
						if (prs.current_block->parent == nullptr) { 
							prs.current_block = prs.default_routine_block;
						} else {
							prs.current_block = prs.current_block->parent;
						}
						reset_instruction(prs);
						break;
					}
					FALLTHROUGH;
				case '\n':
					prs.line++;
					prs.char_in_instruction--;
					break;
				case '\t':
					if (prs.string.state != ParseString::INSIDE && prs.string.state != ParseString::BEGIN) {
						prs.char_in_instruction--;
						break;
					}
					FALLTHROUGH;
				default:
					eval_char(c, prs);
			}

			#ifndef NDEBUG
				prs.debug.evaluated_chars.push_back(c);
			#endif
			prs.last_char = c;
			prs.char_in_instruction++;
		}
	}
};
