#include "command.hh"

namespace mcvm {
	static void show_help_message() {
		OUT(BOLD("Manage mcvm users"));
		// OUT(BOLD("Usage: ") << "mcvm user " << GRAY("[command] [options]"));
		// OUT(BOLD("Commands:"));
		// OUT(ITALIC("add: ") << _ADD_COMMAND_DESCRIPTION);
	}

	void user_command(const unsigned int argc, CommandArgs argv, CommandData& data) {
		ARGC_CHECK(0,);

		data.config.ensure_loaded(data.paths);
	}
};
