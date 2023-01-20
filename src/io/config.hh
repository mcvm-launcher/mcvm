#pragma once
#include "files.hh"
#include "user/user.hh"
#include "data/profile.hh"
#include "package/package.hh"
#include "java/config.hh"

#include <rapidjson/filereadstream.h>
#include <rapidjson/filewritestream.h>
#include <rapidjson/prettywriter.h>

#include <map>

namespace mcvm {
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

	class ProgramConfig {
		bool is_loaded = false;

		void load(const CachedPaths& paths);
		
		public:
		ProgramConfig();

		std::map<std::string, User*> users;
		std::map<std::string, Profile*> profiles;

		User* default_user;

		// Load the config if it isn't loaded already
		void ensure_loaded(const CachedPaths& paths);

		~ProgramConfig() {
			DEL_MAP(profiles);
			DEL_MAP(users);
		}
	};
};
