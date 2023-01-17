#include "config.hh"

// Used for checking that a key is in the config and throwing if it isnt
#define _CONFIG_ENSURE_KEY(obj, obj_name, key) if (!obj.HasMember(key)) { \
	throw ConfigEvalError{config_path, "Expected key [" key "] in " obj_name " object"}; \
}

// Used for checking that a key exists and is of the right type in the config and throwing if it isnt
#define _CONFIG_ENSURE_TYPE(obj, obj_name, key, type) _CONFIG_ENSURE_KEY(obj, obj_name, key); \
if (!obj[key].Is ## type()) { \
	throw ConfigEvalError{config_path, "Key '" key "' in " obj_name " object was expected to be of type '" # type "'"}; \
}

// Put braces around text
#define _BRACIFY(text) "[" + text + "]"

namespace mcvm {
	void write_program_config(json::Document& doc, const CachedPaths& paths) {
		const fs::path config_path = paths.config / "mcvm.json";
		json_write(doc, config_path);
	}

	void open_program_config(json::Document& doc, const fs::path& config_path) {
		if (file_exists(config_path)) {
			json_read(doc, config_path);
		} else {
			doc.SetObject();
			doc.AddMember("users", json::kArrayType, doc.GetAllocator());
			
			json_write(doc, config_path);
		}
	}

	void fetch_program_config(ProgramConfig& config, const CachedPaths& paths) {
		const fs::path config_path = paths.config / "mcvm.json";
		json::Document doc;
		open_program_config(doc, config_path);

		_CONFIG_ENSURE_TYPE(doc, "root", "users", Object);
		if (!doc.HasMember("users")) doc.AddMember("users", json::kObjectType, doc.GetAllocator());
		for (auto& userval : doc["users"].GetObject()) {
			json::GenericObject user_obj = userval.value.GetObject();
			const std::string user_id = userval.name.GetString();

			_CONFIG_ENSURE_TYPE(user_obj, "[user]", "type", String);
			const std::string user_type = user_obj["type"].GetString();

			if (user_type == "microsoft") {
				MicrosoftUser* user;

				_CONFIG_ENSURE_TYPE(user_obj, "[user]", "name", String);
				const std::string name = user_obj["name"].GetString();

				if (user_obj.HasMember("uuid")) {
					_CONFIG_ENSURE_TYPE(user_obj, "[user]", "uuid", String);
					user = new MicrosoftUser(user_id, name, user_obj["uuid"].GetString());
				} else {
					OUT("Warning: It is recommended to have your uuid along with your username in user profile " + name);
					user = new MicrosoftUser(user_id, name);
					user->ensure_uuid();
				}
				config.users.push_back(user);
			} else if (user_type == "demo") {
				config.users.push_back(new DemoUser(user_id));
			} else {
				throw ConfigEvalError{config_path, "Unknown user type '" + user_type + "'."};
			}
		}
	}
};
