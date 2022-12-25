#include "commands/command.hh"
#include "io/files.hh"
#include "net/net.hh"
#include "package/package.hh"

#include <assert.h>
#include <iostream>

int main(int argc, char** argv) {
	mcvm::net_start();

	// Directories
	const fs::path home_dir = mcvm::get_home_dir();
	const fs::path mcvm_dir = mcvm::get_mcvm_dir(home_dir);
	const fs::path cache_dir = mcvm::get_cache_dir(home_dir);
	mcvm::create_dir_if_not_exists(mcvm_dir);
	mcvm::create_dir_if_not_exists(cache_dir);

	mcvm::Profile prof("1.19.3 Vanilla", "1.19.3");
	mcvm::ClientInstance inst(&prof, "1.19.3 Vanilla", mcvm_dir);

	mcvm::RemotePackage* rmt_pkg = new mcvm::RemotePackage("sodium", "localhost:8080/sodium.pkg.txt", cache_dir);
	rmt_pkg->ensure_contents();

	prof.add_package(rmt_pkg);

	// If we have 0-1 args, send the help message
	if (argc <= 1) {
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

	prof.delete_all_packages();

	mcvm::net_stop();

	return 0;
}
