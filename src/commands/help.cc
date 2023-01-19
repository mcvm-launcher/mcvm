#include "command.hh"

namespace mcvm {
	void show_main_help_message() {
		OUT(BOLD("Usage: ") << "mcvm " << GRAY("[subcommand] [...]"));
		OUT_NEWLINE();
		OUT(BOLD("Commands:"));
		OUT("\t" << ITALIC("help: ") << "show this message");
		OUT("\t" << ITALIC("user: ") << "modify users and accounts");
		OUT("\t" << ITALIC("profile: ") << "modify, add, and launch profiles");
		OUT("\t" << ITALIC("launch: ") << "launch instances (play the game!)");
	}

	void help_command(const unsigned int argc, UNUSED CommandArgs argv, CommandData& data) {
		show_main_help_message();
	}
};
