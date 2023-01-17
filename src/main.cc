#include "commands/command.hh"
#include "package/package.hh"
#include "daemon.hh"
#include "io/game.hh"
#include "io/config.hh"

#include <assert.h>
#include <iostream>

inline void run_subcommand(const std::string& subcommand, int argc, std::vector<std::string> argv, const mcvm::CachedPaths& paths) {
	mcvm::command_map.at(subcommand)(argc, argv, paths);
}

int main(int argc, char** argv) {
	mcvm::net_start();

	// Directories
	const mcvm::CachedPaths paths;

	// Config
	mcvm::ProgramConfig config;
	try {
		mcvm::fetch_program_config(config, paths);
	} catch (mcvm::ConfigEvalError& err) {
		OUT(err.what());
		return 1;
	}

	// mcvm::Daemon dmon(paths.run);
	// dmon.ensure_started();

	mcvm::Profile prof("Vanilla", mcvm::MinecraftVersion::V_20W11A);
	mcvm::ClientInstance client(&prof, "Vanilla", paths);
	// mcvm::LocalPackage pkg("sodium", paths.home / "test/sodium2.pkg.txt");
	// pkg.ensure_contents();
	// pkg.parse();
	// mcvm::PkgEvalData res;
	// mcvm::PkgEvalGlobals global;
	// global.mc_version = prof.get_version();
	// global.side = mcvm::MinecraftSide::CLIENT;
	// pkg.evaluate(res, "@install", global);
	client.create(paths);

	// mcvm::MicrosoftUser user("carbon", "CarbonSmasher");
	// client.launch(&user, paths);

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
			run_subcommand(subcommand, argc_slice, argv_slice, paths);
		} catch(const std::out_of_range& e) {
			ERR("Unknown subcommand " << subcommand);
			OUT(mcvm::help_message());
		}
	}

	// prof.delete_all_packages();

	mcvm::net_stop();

	return 0;
}
