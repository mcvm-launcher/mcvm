#pragma once
#include "data/profile.hh"

#include <rapidjson/rapidjson.h>

namespace mcvm {
	// Set of game options that are added to and passed as args before running the game
	class GameRunner {
		// The command that is run with system() to launch the game
		std::string output = "";
		
		public:
		GameRunner(Profile* profile, const MCVersion& _version);

		// Add a command line flag to the command
		void add_flag(const std::string& flag);
		// Finish up and launch
		void launch();

		MCVersion version;
	};
};
