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

	inline void show_update_help_message() {
		OUT_LIT("Update the packages of a profile");
		OUT_LIT("Usage: mcvm profile update [name]");
	}

	inline void profile_update_command(const std::string& name, CommandData& data) {
		if (data.config.profiles.contains(name)) {
			Profile* profile = data.config.profiles[name];
			OUT_LIT("Updating packages...");
			profile->update_packages();
			OUT_LIT("Updating instances...");
			profile->create_instances(data.paths);
		} else {
			ERR("Error: No profile named '" << name << "'.");
		}
	}

	void profile_command(const unsigned int argc, CommandArgs argv, CommandData& data) {	
		ARGC_CHECK(0,);

		if (argv[0] == "add") {
			ARGC_CHECK(1, _add);
		} else if (argv[0] == "update") {
			ARGC_CHECK(1, _update);
			profile_update_command(argv[1], data);
		}
	}
};
