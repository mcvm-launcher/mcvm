#include "commands/command.hh"
#include "io/paths.hh"
#include "net/net.hh"

#include <assert.h>
#include <iostream>

int main(int argc, char** argv) {
	mcvm::net_start();

	try {
		mcvm::obtain_libraries("1.19.3");
	} catch (mcvm::VersionNotFoundException& e) {
		ERR(e.what());
	}

	assert(argc > 0);
	// If we have 1 arg (just the executable), send the help message
	if (argc == 1) {
		std::cout << mcvm::help_message() << "\n";
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
			std::cerr << "Unknown subcommand " << subcommand << "\n";
			std::cout << mcvm::help_message() << "\n";
		}
	}

	mcvm::net_stop();

	return 0;
}
