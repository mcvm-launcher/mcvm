#include "profile.hh"

namespace mcvm {
	Profile::Profile(const std::string _name, MinecraftVersion _version)
	: name(_name), version(_version) {}

	const std::string& Profile::get_name() {
		return name;
	}

	MinecraftVersion Profile::get_version() {
		return version;
	}

	void Profile::add_package(Package* pkg) {
		packages.push_back(pkg);
	}

	void Profile::update_packages() {
		for (uint i = 0; i < packages.size(); i++) {
			Package* pkg = packages[i];
			pkg->ensure_contents();
			pkg->parse();
			mcvm::PkgEvalData res;
			mcvm::PkgEvalGlobals global;
			global.mc_version = version;
			global.side = mcvm::MinecraftSide::CLIENT;
			pkg->evaluate(res, "@install", global);
		}
	}

	void Profile::delete_all_packages() {
		for (auto i = packages.begin(); i != packages.end(); i++) {
			delete *i;
		}
		packages = {};
	}

	void Profile::create_instances(const CachedPaths& paths) {
		for (auto i = instances.begin(); i != instances.end(); i++) {
			OUT(BOLD("Updating instance '" << i->first << "'..."));
			i->second->create(paths);
		}
	}

	Instance::Instance(Profile* _parent, const std::string _name, const CachedPaths& paths, const std::string& subpath)
	: parent(_parent), name(_name), dir(paths.data / subpath / name) {
		parent->instances.insert(std::make_pair(name, this));
	}

	void Instance::create(const CachedPaths& paths, bool verbose) {
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

	void ClientInstance::create(const CachedPaths& paths, bool verbose) {
		ensure_instance_dir();
		const fs::path mc_dir = dir / ".minecraft";

		std::shared_ptr<DownloadHelper> helper = obtain_libraries(
			parent->get_version(),
			&version_json,
			paths,
			verbose
		);

		// Get the client jar
		json::GenericObject client_download = json_access(
			json_access(version_json, "downloads"),
			"client"
		).GetObject();
		const std::string client_url = json_access(client_download, "url").GetString();
		if (verbose) OUT_LIT("\tDownloading client jar");
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

	void ServerInstance::create(const CachedPaths& paths, bool verbose) {
		ensure_instance_dir();
		
		std::shared_ptr<DownloadHelper> helper = obtain_version_json(
			mc_version_reverse_map.at(parent->get_version()),
			&version_json,
			paths,
			verbose
		);

		// Get the server jar
		const fs::path jar_path = server_dir / "server.jar";
		json::GenericObject server_download = json_access(
			json_access(version_json, "downloads"),
			"server"
		).GetObject();

		if (verbose) OUT_LIT("\tDownloading server jar");
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
