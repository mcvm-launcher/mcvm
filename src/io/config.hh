#pragma once
#include "files.hh"
#include "user/user.hh"
#include "data/profile.hh"
#include "package/package.hh"

#include <rapidjson/filereadstream.h>
#include <rapidjson/filewritestream.h>
#include <rapidjson/prettywriter.h>

#include <map>

namespace mcvm {
	struct ProgramConfig {
		std::vector<User*> users;
		std::vector<Profile*> profiles;
		std::vector<Instance*> instances;

		User* default_user = nullptr;

		~ProgramConfig() {
			DEL_VECTOR(profiles);
			DEL_VECTOR(instances);
			DEL_VECTOR(users);
		}
	};

	struct ConfigEvalError : public std::exception {
		const fs::path path;
		const std::string message;

		ConfigEvalError(const fs::path _path, const std::string _message)
		: path(_path), message(_message) {}
		
		std::string what() {
			return NICE_STR_CAT(
				"Error when evaluating config file " + path.c_str() + ":\n"
				+ '\t' + message
			);
		}
	};
	
	// Write JSON data to the program config
	extern void write_program_config(json::Document& doc, const CachedPaths& paths);

	// Open the program config
	extern void open_program_config(json::Document& doc, const fs::path& config_path);

	// Get updated program config
	extern void fetch_program_config(ProgramConfig& config, const CachedPaths& paths);
};
