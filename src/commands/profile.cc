#include "command.hh"

#define _UPDATE_HELP_MESSAGE "Update the packages and instances of a profile"
#define _REINSTALL_HELP_MESSAGE "Force the reinstall of all files for a profile"

namespace mcvm {
	static void show_help_message() {
		OUT_LIT("Manage mcvm profiles");
		OUT(BOLD("Usage: ") << "mcvm profile " << GRAY("[command] [options]"));
		OUT_NEWLINE();
		OUT(BOLD("Commands:"));
		OUT("\t" << ITALIC("update: ") << _UPDATE_HELP_MESSAGE);
		OUT("\t" << ITALIC("reinstall: ") << _REINSTALL_HELP_MESSAGE);
	}

	static void show_update_help_message() {
		OUT(BOLD(_UPDATE_HELP_MESSAGE));
		OUT_NEWLINE();
		OUT(BOLD("Usage: ") << "mcvm profile update " << GRAY("[profile_name]"));
	}

	static void show_reinstall_help_message() {
		OUT(BOLD(_REINSTALL_HELP_MESSAGE));
		OUT_NEWLINE();
		OUT(BOLD("Usage: ") << "mcvm profile reinstall " << GRAY("[profile_name]"));
	}

	static void profile_update_command(const std::string& name, CommandData& data, bool force) {
		data.config.ensure_loaded(data.paths);
		if (data.config.profiles.contains(name)) {
			Profile* profile = data.config.profiles[name];
			OUT_LIT("Updating packages...");
			profile->update_packages();
			OUT_LIT("Updating instances...");
			profile->create_instances(data.paths, force);
		} else {
			ERR("Error: No profile named '" << name << "'.");
		}
	}

	void profile_command(const unsigned int argc, CommandArgs argv, CommandData& data) {	
		ARGC_CHECK(0,);

		if (argv[0] == "update") {
			ARGC_CHECK(1, _update);
			profile_update_command(argv[1], data, false);
		} else if (argv[0] == "reinstall") {
			ARGC_CHECK(1, _reinstall);
			profile_update_command(argv[1], data, true);
		} else {
			ERR("Unknown subcommand '" << argv[0] << "'.");
			return;
		}
	}
};
