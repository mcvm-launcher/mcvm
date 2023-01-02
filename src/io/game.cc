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

	void GameRunner::add_flag(const std::string& flag) {
		flags.push_back(flag);
	}

	bool GameRunner::repl_arg_token(std::string& contents, bool is_jvm)	{
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
			_GAME_ARG_REPL("${assets_root}", get_mcvm_dir() / "assets");
			_GAME_ARG_REPL("${assets_index_name}", get_mcvm_dir() / "assets" / "indexes" / (version + ".json"));
			// TODO: Actual auth
			_GAME_ARG_REPL("${auth_player_name}", "CarbonSmasher");
			_GAME_ARG_REPL("${auth_access_token}", "abc123abc123");
			_GAME_ARG_REPL("${auth_uuid}", "aaaaa-aaaaa-aaaa-a");
		}
		// assert(!contents.find('$'));
		if (contents.find('$')) {
			return true;
		}
		return false;
	}

	void GameRunner::parse_single_arg(const json::Value& arg, bool is_jvm) {
		std::string contents; // The contents of the argument, will get changed based on the json item type and text replacement
		if (arg.IsString()) {
			contents = arg.GetString();
		}
		if (repl_arg_token(contents, is_jvm)) {
			if (flags.size() > 0) flags.pop_back();
			return;
		}
		add_flag(contents);
	}

	void GameRunner::parse_args(json::Document* ret) {
		assert(ret->IsObject());
		assert(ret->HasMember("arguments"));
		json::GenericArray game_args = ret->operator[]("arguments")["game"].GetArray();
		json::GenericArray jvm_args = ret->operator[]("arguments")["jvm"].GetArray();

		for (auto& arg : jvm_args) {
			parse_single_arg(arg, true);
		}
		write_flags();

		assert(ret->HasMember("mainClass"));
		const std::string main_class = ret->operator[]("mainClass").GetString();
		output.push_back(' ');
		output.append(main_class);
		
		for (auto& arg : game_args) {
			parse_single_arg(arg, false);
		}
		write_flags();
	}

	void GameRunner::write_flags() {
		for (unsigned int i = 0; i < flags.size(); i++) {
			output.push_back(' ');
			output.append(flags[i]);
		}
		flags = {};
	}

	void GameRunner::authenticate(const std::string& username, const std::string& access_token) {

	}

	void GameRunner::launch() {
		// system(output.c_str());
		output.push_back(' ');
		output.append(jar_path);
		OUT(output);
	}
};
