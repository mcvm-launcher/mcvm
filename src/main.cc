#include "commands/command.hh"
#include "io/paths.hh"
#include "net/net.hh"

#include <assert.h>
#include <iostream>

int main(int argc, char** argv) {
	mcvm::net_start();

	assert(argc > 0);
	// If we have 1 arg (just the executable), send the help message
	if (argc == 1) {
		OUT(mcvm::help_message());
	}
	// If we have 2+ args (executable and a subcommand, plus any number of args), run the command
	if (argc > 1) {
		const char* subcommand = argv[1];
		// Remove both the executable and subcommand arguments
		const int argc_slice = argc - 2;
		std::vector<std::string> argv_slice;
		for (int i = 2; i < argc; i++) {
			argv_slice.push_back(std::string(argv[i]));
		}

		try {
			mcvm::command_map.at(subcommand)(argc_slice, argv_slice);
		} catch(const std::out_of_range& e) {
			ERR("Unknown subcommand " << subcommand);
			OUT(mcvm::help_message());
		}
	}

	mcvm::net_stop();

	return 0;
}
