#include "command.hh"

namespace mcvm {
	const std::string help_message() {
		return
			"Usage: mcvm [subcommand] [...]" "\n"
			"Commands:" "\n"
			"\t" "help: show this message" "\n"
			"\t" "user: modify users and accounts" "\n"
			"\t" "profile: modify, add, and launch profiles" "\n"
			"\t" "launch: launch instances (play the game!)";
	}

	void help_command(const unsigned int argc, UNUSED CommandArgs argv, CommandData& data) {
		OUT(help_message());
	}
};
