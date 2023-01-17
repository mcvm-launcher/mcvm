#include "command.hh"

#define _ADD_COMMAND_DESCRIPTION "Create a new user"

namespace mcvm {
	inline void show_help_message() {
		OUT_LIT("Manage mcvm users");
		OUT_LIT("Usage: mcvm user [command] [options]");
		OUT_LIT("Commands:");
		OUT_LIT("add: " _ADD_COMMAND_DESCRIPTION);
	}

	inline void show_add_help_message() {
		OUT_LIT(_ADD_COMMAND_DESCRIPTION);
		OUT_LIT("Usage: mcvm user add [name]");
	}

	inline void _add_command(const std::string& id, const std::string& name, const CachedPaths& paths) {
		// json::Document doc;
		// const fs::path config_path = paths.config / "mcvm.json";
		// open_program_config(doc, config_path);
		// json::GenericObject users = doc["users"].GetObject();
		// json::Value user(json::kObjectType);
		// user.AddMember("name", name, doc.GetAllocator());
		// user.AddMember("type", "microsoft", doc.GetAllocator());
		// users.AddMember(json::StringRef(id.c_str()), user, doc.GetAllocator());

		// write_program_config(doc, paths);
	}

	void user_command(const unsigned int argc, CommandArgs argv, const CachedPaths& paths) {
		ARGC_CHECK(0,);

		if (argv[0] == "add") {
			ARGC_CHECK(2, _add);
			_add_command(argv[1], argv[2], paths);
		}
	}
};
