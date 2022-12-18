#include "command.hh"

namespace mcvm {
	const std::string help_message() {
		// TODO: Add help message
		return "help help help";
	}

	void help_command(const unsigned int argc, CommandArgs argv) {
		std::cout << help_message() << "\n";
	}
};
