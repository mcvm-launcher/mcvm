#include "config.hh"

// Used for checking that a key is in the config and throwing if it isnt
#define _CONFIG_ENSURE_KEY(obj, obj_name, key) if (!obj.HasMember(key)) { \
	throw ConfigEvalError{config_path, "Expected key [" key "] in " obj_name " object"}; \
}

// Used for checking that a key is of the right type in the config and throwing if it isnt
#define _CONFIG_ENSURE_TYPE(obj, obj_name, key, type) if (!obj[key].Is ## type()) { \
	throw ConfigEvalError{config_path, "Key '" key "' in " obj_name " object was expected to be of type '" # type "'"}; \
}

// Put braces around text
#define _BRACIFY(text) "[" + text + "]"

namespace mcvm {
	void fetch_program_config(ProgramConfig& config, const CachedPaths& paths) {
		const fs::path config_path = paths.config / "mcvm.json";

		json::Document doc;
		FILE* config_file;
		if (file_exists(config_path)) {
			config_file = fopen(config_path.c_str(), "rb");
			char readbuf[CHARBUF_LARGE];
			json::FileReadStream st(config_file, readbuf, sizeof(readbuf));
			json::AutoUTFInputStream<unsigned, json::FileReadStream> ist(st);
			doc.ParseStream<0, json::AutoUTF<unsigned>>(ist);
		} else {
			doc.SetObject();
			doc.AddMember("users", json::kArrayType, doc.GetAllocator());
			
			config_file = fopen(config_path.c_str(), "wb");
			char writebuf[CHARBUF_LARGE];
			json::FileWriteStream os(config_file, writebuf, sizeof(writebuf));
			
			json::PrettyWriter writer(os);
			writer.SetIndent('\t', 1);
			doc.Accept(writer);
		}

		_CONFIG_ENSURE_TYPE(doc, "root", "users", Object);
		if (!doc.HasMember("users")) doc.AddMember("users", json::kObjectType, doc.GetAllocator());
		for (auto& userval : doc["users"].GetObject()) {
			json::GenericObject user_obj = userval.value.GetObject();
			const std::string user_id = userval.name.GetString();

			_CONFIG_ENSURE_KEY(user_obj, "[user]", "type");
			_CONFIG_ENSURE_TYPE(user_obj, "[user]", "type", String);
			const std::string user_type = user_obj["type"].GetString();

			if (user_type == "microsoft") {
				_CONFIG_ENSURE_KEY(user_obj, "[user]", "name");
				_CONFIG_ENSURE_TYPE(user_obj, "[user]", "name", String);
				const std::string name = user_obj["name"].GetString();
				config.users.push_back(new MicrosoftUser(user_id, name));
			} else if (user_type == "demo") {
				config.users.push_back(new DemoUser(user_id));
			} else {
				throw ConfigEvalError{config_path, "Unknown user type '" + user_type + "'."};
			}
		}

		fclose(config_file);
	}
};
