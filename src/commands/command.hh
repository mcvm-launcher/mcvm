#pragma once
#include "data/profile.hh"
#include "user/user.hh"
#include "io/files.hh"
#include "lib/util.hh"
#include "daemon.hh"

#include <map>
#include <vector>

// Check if argc is a certain value. If not, then show the help message for the command
#define ARGC_CHECK(len, subcommand) if (argc == len) { show ## subcommand ## _help_message(); return; }

namespace mcvm {
	typedef std::vector<std::string>& CommandArgs;

	// Command function definitions
	extern void user_command(const unsigned int argc, CommandArgs argv);
	extern void profile_command(const unsigned int argc, CommandArgs argv);
	extern void help_command(const unsigned int argc, CommandArgs argv);
	// Internal command used as the init function for the daemon
	static void start_daemon_command(UNUSED const unsigned int argc, UNUSED CommandArgs argv) {
		Daemon::daemon_init();
	}

	// Command subfunction definitions
	extern const std::string help_message();

	static std::map<std::string, void(*)(unsigned int, CommandArgs)> command_map = {
		{"user", &user_command},
		{"profile", &profile_command},
		{"help", &help_command},
		{"__daemon_start__", &start_daemon_command}
	};
};
