#include "ast.hh"

namespace mcvm {
	void new_instruction(ParseData& prs) {
		prs.instruction++;
		prs.char_in_instruction = -1;

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
				if (prs.root.words.at(1) == "match") {
					cond.condition = PkgIfCondition::MATCH;
				}
				if (prs.root.words.at(1) == "version") {
					cond.condition = PkgIfCondition::VERSION;
				}
				if (prs.root.words.at(1) == "modloader") {
					cond.condition = PkgIfCondition::MODLOADER;
				}

				cond.left_side = prs.root.words.at(1);
				if (prs.root.words.size() > 3) {
					cond.right_side = prs.root.words.at(3);
				}
				inst->condition = cond;
				inst->nested_block = new PkgBlock;
				inst->nested_block->parent = prs.current_block;
				prs.current_block->instructions.push_back(inst);
				prs.current_block = inst->nested_block;
			} else if (instruction == "endif") {
				// We will need to check that this is an if block eventually
				prs.current_block = prs.current_block->parent;
			} else {
				PkgCommandInstruction* inst = new PkgCommandInstruction;
				inst->command = instruction;
				inst->args = vec_slice(prs.root.words, 1, prs.root.words.size() - 1);
				prs.current_block->instructions.push_back(inst);
			}
		}
		prs.root.words = { "" };
		prs.root.is_routine = false;
	}

	void parse_root(const char& c, ParseData& prs) {
		switch (c) {
			case ' ':
				prs.root.words.push_back("");
				prs.root.is_routine = false;
				break;
			case '@':
				FALLTHROUGH if (prs.char_in_instruction == 0) {
					prs.root.is_routine = true;
				}
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

	PkgAST* Package::parse() {
		ParseData prs;
		PkgAST* ast = new PkgAST;
		prs.ast = ast;
		auto pair = ast->routines.insert(std::make_pair("__default", PkgBlock{}));
		prs.current_block = &pair.first->second;
		
		for (unsigned int i = 0; i < contents.length(); i++) {
			prs.character = i;
			const char& c = contents[i];

			switch (c) {
				case ';':
					FALLTHROUGH if (prs.string.state != ParseString::INSIDE && prs.string.state != ParseString::BEGIN) {
						new_instruction(prs);
						break;
					}
				case '\n':
					prs.line++;
					new_instruction(prs);
					break;
				default:
					eval_char(c, prs);
			}

			#ifndef NDEBUG
				prs.debug.evaluated_chars.push_back(c);
			#endif
			prs.last_char = c;
			prs.char_in_instruction++;
		}

		return ast;
	}
};
