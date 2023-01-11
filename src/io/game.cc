#include "game.hh"

namespace mcvm {
	// README: https://wiki.vg/Launching_the_game
	// FIXME

	GameRunner::GameRunner(
		const MCVersion& _version,
		const fs::path _mc_dir,
		const fs::path _jar_path,
		User* _user
	)
	: version(_version), mc_dir(_mc_dir), jar_path(_jar_path), user(_user) {}

	void GameRunner::add_word(const std::string& word) {
		output.push_back(' ');
		output.append(word);
	}

	void GameRunner::add_flag(const std::string& flag) {
		flags.push_back(flag);
	}

	bool GameRunner::repl_arg_token(std::string& contents, bool is_jvm, const CachedPaths& paths)	{
		if (is_jvm) {
			fandr(contents, "${launcher_name}", "mcvm");
			fandr(contents, "${launcher_version}", "alpha");
		} else {
			#define _GAME_ARG_REPL(check, expr) if (contents == check) contents = expr

			// Version
			_GAME_ARG_REPL("${version_name}", version);
			_GAME_ARG_REPL("${version_type}", "mcvm");
			// Directories
			_GAME_ARG_REPL("${game_directory}", mc_dir);
			_GAME_ARG_REPL("${assets_root}", paths.assets);
			_GAME_ARG_REPL("${assets_index_name}", paths.assets / "indexes" / (version + ".json"));
			// TODO: Actual auth
			_GAME_ARG_REPL("${auth_player_name}", "CarbonSmasher");
			_GAME_ARG_REPL("${auth_access_token}", "abc123abc123");
			_GAME_ARG_REPL("${auth_uuid}", "aaaaa-aaaaa-aaaa-a");
		}
		// assert(!contents.find('$'));
		if (contents.find('$') != std::string::npos) {
			// return true;
		}
		return false;
	}

	void GameRunner::parse_single_arg(json::Value& arg, bool is_jvm, const CachedPaths& paths) {
		// The contents of the argument, will get changed based on the json item type and text replacement
		std::string contents;
		if (arg.IsString()) {
			contents = arg.GetString();
		} else if (arg.IsObject()) {
			json::GenericObject arg_obj = arg.GetObject();
			json::GenericArray rules = json_access(arg_obj, "rules").GetArray();
			for (auto& rule_val : rules) {
				json::GenericObject rule = rule_val.GetObject();
				const bool allowed = is_allowed(json_access(rule, "action").GetString());
				if (rule.HasMember("os")) {
					json::GenericObject os = rule["os"].GetObject();
					if (os.HasMember("name")) {
						if (allowed != (OS_STRING == os["name"])) return;
					}
					if (os.HasMember("arch")) {
						if (allowed != (ARCH_STRING == os["arch"])) return;
					}
				}
				if (rule.HasMember("features")) {
					json::GenericObject features = rule["features"].GetObject();
					if (features.HasMember("has_custom_resolution")) {
						return;
					}
					if (features.HasMember("is_demo_user")) {
						return;
					}
				}
			}
			parse_single_arg(json_access(arg_obj, "value"), is_jvm, paths);
			return;
		} else if (arg.IsArray()) {
			for (auto& value : arg.GetArray()) {
				parse_single_arg(value, is_jvm, paths);
			}
		} else {
			ASSERT_NOREACH();
		}
		if (repl_arg_token(contents, is_jvm, paths)) {
			if (flags.size() > 0) flags.pop_back();
			return;
		}
		assert(contents != "");
		add_flag(contents);
	}

	void GameRunner::parse_args(json::Document* ret, const CachedPaths& paths) {
		assert(ret->IsObject());
		json::GenericObject arguments = json_access(ret, "arguments").GetObject();
		json::GenericArray game_args = json_access(arguments, "game").GetArray();
		json::GenericArray jvm_args = json_access(arguments, "jvm").GetArray();

		for (auto& arg : jvm_args) {
			parse_single_arg(arg, true, paths);
		}
		write_flags();

		const std::string main_class = json_access(ret, "mainClass").GetString();
		add_word(main_class);
		
		for (auto& arg : game_args) {
			parse_single_arg(arg, false, paths);
		}
		write_flags();
	}

	void GameRunner::write_flags() {
		for (uint i = 0; i < flags.size(); i++) {
			add_word(flags[i]);
		}
		flags = {};
	}

	void GameRunner::launch() {
		// system(output.c_str());
		add_word(jar_path);
		OUT(output);
	}
};
