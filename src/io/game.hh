#pragma once
#include "data/profile.hh"
#include "paths.hh"

namespace mcvm {
	// Set of game options that are added to and passed as args before running the game
	class GameRunner {
		std::string output = "";
		
		public:
		GameRunner(Profile* profile, const MCVersion& _version);

		void add_flag(const std::string& flag);

		MCVersion version;
	};
};
