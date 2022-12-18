#pragma once
#include "data/profile.hh"
#include "user/user.hh"

#include <map>
#include <vector>
#include <iostream>

namespace mcvm {
	typedef const std::vector<std::string>& CommandArgs;

	// Command function definitions
	extern void user_command(const unsigned int argc, CommandArgs argv);
	extern void profile_command(const unsigned int argc, CommandArgs argv);
	extern void help_command(const unsigned int argc, CommandArgs argv);

	// Command subfunction definitions
	extern const std::string help_message();

	static std::map<std::string, void(*)(unsigned int, CommandArgs)> command_map = {
		{"user", &user_command},
		{"profile", &profile_command},
		{"help", &help_command}
	};
};
