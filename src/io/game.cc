#include "game.hh"

namespace mcvm {
	GameRunner::GameRunner(Profile* profile, const MCVersion& _version)
	: version(_version) {
		
	}

	void GameRunner::launch() {
		system(output);
	}
};
