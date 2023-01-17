#include "profile.hh"

namespace mcvm {
	Profile::Profile(const std::string _name, MinecraftVersion _version)
	: name(_name), version(_version) {}

	MinecraftVersion Profile::get_version() {
		return version;
	}

	void Profile::add_package(Package* pkg) {
		packages.push_back(pkg);
	}

	void Profile::delete_all_packages() {
		for (auto i = packages.begin(); i != packages.end(); i++) {
			delete *i;
		}
		packages = {};
	}

	Instance::Instance(Profile* _parent, const std::string _name, const CachedPaths& paths, const std::string& subpath)
	: parent(_parent), name(_name), dir(paths.data / subpath / name) {}

	void Instance::create(const CachedPaths& paths) {
		ensure_instance_dir();
	}

	void Instance::ensure_instance_dir() {
		create_leading_directories(dir);
		create_dir_if_not_exists(dir);
	}
	
	MinecraftVersion Instance::get_version() {
		return parent->get_version();
	}

	ClientInstance::ClientInstance(Profile* _parent, const std::string _name, const CachedPaths& paths)
	: Instance(_parent, _name, paths, CLIENT_INSTANCES_DIR) {}

	void ClientInstance::create(const CachedPaths& paths) {
		ensure_instance_dir();
		const fs::path mc_dir = dir / ".minecraft";

		std::shared_ptr<DownloadHelper> helper = obtain_libraries(parent->get_version(), &version_json, paths);

		// Get the client jar
		json::GenericObject client_download = json_access(
			json_access(version_json, "downloads"),
			"client"
		).GetObject();
		const std::string client_url = json_access(client_download, "url").GetString();
		OUT_LIT("Downloading client jar");
		download_cached_file(client_url, dir / "client.jar", false, helper);
	}

	void ClientInstance::ensure_instance_dir() {
		Instance::ensure_instance_dir();
		const fs::path mc_dir = dir / ".minecraft";
		create_dir_if_not_exists(mc_dir);
		create_dir_if_not_exists(mc_dir / "assets");
	}

	void ClientInstance::launch(User* user, const CachedPaths& paths) {
		mcvm::GameRunner game(parent->get_version(), dir / ".minecraft", dir / "client.jar", user);
		game.parse_args(&version_json, paths);
		game.launch();
	}

	ServerInstance::ServerInstance(Profile* _parent, const std::string _name, const CachedPaths& paths)
	: Instance(_parent, _name, paths, SERVER_INSTANCES_DIR), server_dir(dir / "server") {}

	void ServerInstance::create(const CachedPaths& paths) {
		ensure_instance_dir();
		
		std::shared_ptr<DownloadHelper> helper = obtain_version_json(mc_version_reverse_map.at(parent->get_version()), &version_json, paths);

		// Get the server jar
		const fs::path jar_path = server_dir / "server.jar";
		json::GenericObject server_download = json_access(
			json_access(version_json, "downloads"),
			"server"
		).GetObject();

		download_cached_file(
			json_access(server_download, "url").GetString(),
			jar_path, false, helper
		);

		// Create the EULA
		write_file(server_dir / "eula.txt", "eula = true\n");
	}

	void ServerInstance::ensure_instance_dir() {
		Instance::ensure_instance_dir();
		create_dir_if_not_exists(dir / "server");
	}
};
