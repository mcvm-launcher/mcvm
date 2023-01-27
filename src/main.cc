#include "commands/command.hh"
#include "package/package.hh"
#include "daemon.hh"
#include "io/config.hh"

#include <assert.h>
#include <iostream>

inline void run_subcommand(
	const std::string& subcommand,
	int argc, std::vector<std::string> argv,
	mcvm::CommandData& data
) {
	try {
		assert(argc == argv.size());
		mcvm::command_map.at(subcommand)(argc, argv, data);
	} catch (mcvm::FileOpenError& err) {
		ERR(err.what());
		exit(1);
	}
}

int main(int argc, char** argv) {
	mcvm::net_start();

	// Directories
	const mcvm::CachedPaths paths;

	// Config
	mcvm::ProgramConfig config;

	mcvm::CommandData command_data{paths, config};

	run_subcommand("launch", 2, {"1.19", "server"}, command_data);

	// mcvm::Daemon dmon(paths.run);
	// dmon.ensure_started();

	// If we have 0-1 args, send the help message
	if (argc <= 1) {
		mcvm::show_main_help_message();
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
			run_subcommand(subcommand, argc_slice, argv_slice, command_data);
		} catch(const std::out_of_range& e) {
			ERR("Unknown subcommand " << subcommand);
			mcvm::show_main_help_message();
		}
	}

	mcvm::net_stop();

	return 0;
}
