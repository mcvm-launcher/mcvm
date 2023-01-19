#include "config.hh"

// Used for checking that a key is in the config and throwing if it isnt
#define _CONFIG_ENSURE_KEY(obj, obj_name, key) if (!obj.HasMember(key)) { \
	throw ConfigEvalError{config_path, "Expected key [" key "] in " obj_name " object"}; \
}

// Used for checking that a key is of the right type in the config and throwing if it isnt
#define _CONFIG_ENSURE_TYPE(obj, obj_name, key, type) if (!obj[key].Is ## type()) { \
	throw ConfigEvalError{config_path, "Key '" key "' in " obj_name " object was expected to be of type '" # type "'"}; \
}

// Used for checking that a key exists and is of the right type in the config and throwing if it isnt
#define _CONFIG_ENSURE(obj, obj_name, key, type) _CONFIG_ENSURE_KEY(obj, obj_name, key); \
	_CONFIG_ENSURE_TYPE(obj, obj_name, key, type)

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

		// Users
		_CONFIG_ENSURE(doc, "root", "users", Object);
		if (!doc.HasMember("users")) doc.AddMember("users", json::kObjectType, doc.GetAllocator());
		for (auto& user_val : doc["users"].GetObject()) {
			const std::string user_id = user_val.name.GetString();
			json::GenericObject user_obj = user_val.value.GetObject();

			_CONFIG_ENSURE(user_obj, "[user]", "type", String);
			const std::string user_type = user_obj["type"].GetString();

			if (user_type == "microsoft") {
				MicrosoftUser* user;

				_CONFIG_ENSURE(user_obj, "[user]", "name", String);
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

		if (doc.HasMember("default_user")) {
			_CONFIG_ENSURE_TYPE(doc, "root", "default_user", String)
			for (uint i = 0; i < config.users.size(); i++) {
				User* user = config.users[i];
				if (user->id == doc["default_user"].GetString()) {
					config.default_user = user;
				}
			}
		}

		// Profiles
		_CONFIG_ENSURE(doc, "root", "profiles", Object);
		for (auto& profile_val : doc["profiles"].GetObject()) {
			const std::string profile_name = profile_val.name.GetString();
			json::GenericObject profile_obj = profile_val.value.GetObject();

			_CONFIG_ENSURE(profile_obj, "[profile]", "version", String);
			const std::string profile_version_str = profile_obj["version"].GetString();
			MinecraftVersion profile_version;
			if (mc_version_forward_map.count(profile_version_str)) {
				profile_version = mc_version_forward_map[profile_version_str];
			} else {
				throw ConfigEvalError{config_path, "Invalid Minecraft version '" + profile_version_str + "'."};
			}

			Profile* profile = new Profile(profile_name, profile_version);
			config.profiles.push_back(profile);

			// Instances
			if (profile_obj.HasMember("instances")) {
				_CONFIG_ENSURE_TYPE(profile_obj, "[profile]", "instances", Object);
				for (auto& instance_val : profile_obj["instances"].GetObject()) {
					const std::string instance_name = instance_val.name.GetString();
					json::GenericObject instance_obj = instance_val.value.GetObject();

					_CONFIG_ENSURE_TYPE(instance_obj, "[profile][instance]", "type", String);
					const std::string instance_type = instance_obj["type"].GetString();
					Instance* instance;
					if (instance_type == "client") {
						instance = new ClientInstance(profile, instance_name, paths);
					} else if (instance_type == "server") {
						instance = new ServerInstance(profile, instance_name, paths);
					} else {
						throw ConfigEvalError{config_path, "Unknown instance type '" + instance_type + "'."};
					}

					config.instances.push_back(instance);
				}
			}

			// Packages
			if (profile_obj.HasMember("packages")) {
				_CONFIG_ENSURE_TYPE(profile_obj, "[profile]", "packages", Array);
				for (auto& package_val : profile_obj["packages"].GetArray()) {
					json::GenericObject package_obj = package_val.GetObject();

					_CONFIG_ENSURE_TYPE(package_obj, "[profile][package]", "type", String);
					const std::string package_type = package_obj["type"].GetString();
					Package* package;
					if (package_type == "local") {
						_CONFIG_ENSURE_TYPE(package_obj, "[profile][package]", "path", String);
						std::string package_path_str = package_obj["path"].GetString();
						const fs::path package_path = substitute_home(package_path_str, paths);
						const std::string package_name = package_path.stem();
						package = new LocalPackage(package_name, package_path);
					} else if (package_type == "remote") {
						_CONFIG_ENSURE_TYPE(package_obj, "[profile][package]", "url", String);
						const std::string package_url = package_obj["url"].GetString();
						// TODO
					} else {
						throw ConfigEvalError{config_path, "Unknown package type '" + package_type + "'."};
					}

					profile->add_package(package);
				}
			}
		}
	}
};
