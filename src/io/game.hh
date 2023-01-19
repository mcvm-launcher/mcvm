#pragma once
#include "data/resource.hh"
#include "user/user.hh"
#include "net/net.hh"
#include "lib/json.hh"
#include "lib/mojang.hh"

#include <rapidjson/rapidjson.h>

#include <deque>

namespace mcvm {
	// Set of game options that are added to and passed as args before running the game
	class GameRunner {
		// The command that is run with system() to launch the game
		std::string output = "java";
		// The list of flags to be written and appended to the output
		// TODO: Make this a stack probably
		std::deque<std::string> flags;

		// Properties
		const MinecraftVersion version;
		const fs::path mc_dir;
		const fs::path jar_path;
		User* user;
		const std::string& classpath;

		// Replaces tokens on an argument. Returns true if the previous argument should be deleted
		bool repl_arg_token(std::string& contents, bool is_jvm, const CachedPaths& paths);
		// Parse a single argument value from the document
		void parse_single_arg(json::Value& arg, bool is_jvm, const CachedPaths& paths);
		// Write flags to the output
		void write_flags();
		
		// Add word to the command
		void add_word(const std::string& word);
		// Add a command line flag to the command
		void add_flag(const std::string& flag);

		public:
		GameRunner(
			MinecraftVersion _version,
			const fs::path _mc_dir,
			const fs::path _jar_path,
			User* _user,
			const std::string& _classpath
		);

		// Parse arguments from a JSON file
		void parse_args(json::Document* ret, const CachedPaths& paths);

		// Finish up and launch
		void launch();
	};
};
