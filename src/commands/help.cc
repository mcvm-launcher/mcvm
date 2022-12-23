#include "command.hh"

namespace mcvm {
	const std::string help_message() {
		return
			"Usage: mcvm [subcommand] [...]" "\n"
			"Commands:" "\n"
			"\t" "help: show this message" "\n"
			"\t" "user: modify users and accounts" "\n"
			"\t" "profile: modify, add, and launch profiles";
	}

	void help_command(const unsigned int argc, CommandArgs argv) {
		std::cout << help_message() << "\n";
	}
};
