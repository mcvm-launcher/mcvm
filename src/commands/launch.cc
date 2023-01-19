#include "command.hh"

namespace mcvm {
	static void show_help_message() {
		OUT(BOLD("Launch the game"));
		OUT(BOLD("Usage: ") << "mcvm launch " << GRAY("[profile] [instance]"));
	}

	void launch_command(const unsigned int argc, CommandArgs argv, CommandData& data) {	
		ARGC_CHECK(0,);
		ARGC_CHECK(1,);

		const std::string& profile_id = argv[0];
		const std::string& instance_id = argv[1];

		Profile* profile = nullptr;
		Instance* instance = nullptr;

		if (data.config.profiles.contains(profile_id)) {
			profile = data.config.profiles[profile_id];
		} else {
			ERR("Error: No profile named '" << profile_id << "'.");
			return;
		}

		if (profile->instances.contains(instance_id)) {
			instance = profile->instances[instance_id];
		} else {
			ERR("Error: No instance named '" << instance_id << "' in profile '" << profile_id << "'." );
			return;
		}

		OUT_LIT("Getting instance ready...");
		instance->create(data.paths, false);

		OUT("Launching instance '" << instance->name << "'.");
		instance->launch(data.config.default_user, data.paths);
	}
};
