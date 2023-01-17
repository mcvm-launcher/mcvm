#include "command.hh"

namespace mcvm {
	inline void show_help_message() {
		OUT_LIT("Manage mcvm profiles");
		OUT_LIT("Usage: mcvm profile [command] [options]");
		OUT_LIT("Commands:");
		OUT_LIT("add: Create a new profile");
	}

	inline void show_add_help_message() {
		OUT_LIT("Create a new profile");
		OUT_LIT("Usage: mcvm profile add [name]");
	}

	void profile_command(const unsigned int argc, CommandArgs argv, const CachedPaths& paths) {	
		ARGC_CHECK(0,);

		if (argv[0] == "add") {
			ARGC_CHECK(1, _add);
		}
	}
};
