#pragma once
#include "data/resource.hh"
#include "user/user.hh"
#include "net/net.hh"
#include "lib/util.hh"

#include <rapidjson/rapidjson.h>

namespace mcvm {
	// Set of game options that are added to and passed as args before running the game
	class GameRunner {
		// The command that is run with system() to launch the game
		std::string output = "java -jar";
		// The list of flags to be written and appended to the output
		// TODO: Make this a stack probably
		std::vector<std::string> flags;

		// Properties
		const MCVersion& version;
		const fs::path mc_dir;
		const fs::path jar_path;
		User* user;

		// Replaces tokens on an argument. Returns true if the previous argument should be deleted
		bool repl_arg_token(std::string& contents, bool is_jvm);
		// Parse a single argument value from the document
		void parse_single_arg(const json::Value& arg, bool is_jvm);
		// Write flags to the output
		void write_flags();
		
		public:
		GameRunner(
			const MCVersion& _version,
			const fs::path _mc_dir,
			const fs::path _jar_path,
			User* _user
		);

		// Add a command line flag to the command
		void add_flag(const std::string& flag);
		// Parse arguments from a JSON file
		void parse_args(json::Document* ret);

		// Options

		void authenticate(const std::string& username, const std::string& access_token);

		// Finish up and launch
		void launch();

	};
};
