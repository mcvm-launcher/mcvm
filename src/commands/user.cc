#include "command.hh"

namespace mcvm {
	static void show_help_message() {
		OUT(BOLD("Manage mcvm users"));
		// OUT(BOLD("Usage: ") << "mcvm user " << GRAY("[command] [options]"));
		// OUT(BOLD("Commands:"));
		// OUT(ITALIC("add: ") << _ADD_COMMAND_DESCRIPTION);
	}

	inline void _add_command(const std::string& id, const std::string& name, CommandData& data) {
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

	void user_command(const unsigned int argc, CommandArgs argv, CommandData& data) {
		ARGC_CHECK(0,);

		// if (argv[0] == "add") {
		// 	ARGC_CHECK(2, _add);
		// 	_add_command(argv[1], argv[2], data);
		// } else {
		// 	ERR("Unknown subcommand '" << argv[0] << "'.");
		// }
	}
};
