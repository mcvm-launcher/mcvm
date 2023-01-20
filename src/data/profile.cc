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

	void Profile::add_package(std::shared_ptr<Package> pkg) {
		packages.push_back(pkg);
	}

	void Profile::update_packages() {
		for (uint i = 0; i < packages.size(); i++) {
			std::shared_ptr<Package> pkg = packages[i];
			pkg->ensure_contents();
			pkg->parse();
			mcvm::PkgEvalData res;
			mcvm::PkgEvalGlobals global;
			global.mc_version = version;
			global.side = mcvm::MinecraftSide::CLIENT;
			pkg->evaluate(res, "@install", global);
		}
	}

	void Profile::create_instances(const CachedPaths& paths) {
		for (auto i = instances.begin(); i != instances.end(); i++) {
			OUT(BOLD("Updating instance '" << i->first << "'..."));
			i->second->create(paths);
		}
	}

	Profile::~Profile() {
		DEL_MAP(instances);
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
		const fs::path jar_path = dir / "client.jar";

		try {
			std::shared_ptr<DownloadHelper> helper = obtain_libraries(
				parent->get_version(),
				&version_json,
				paths,
				classpath,
				verbose
			);

			classpath += jar_path.c_str();

			const std::string major_version = std::to_string(json_access(
				json_access(version_json, "javaVersion"), "majorVersion"
			).GetUint());
			java = new AdoptiumJava(
				major_version
			);

			java->ensure_installed(paths);

			// Get the client jar
			json::GenericObject client_download = json_access(
				json_access(version_json, "downloads"),
				"client"
			).GetObject();
			const std::string client_url = json_access(client_download, "url").GetString();
			if (verbose) OUT_LIT("\tDownloading client jar...");
			download_cached_file(client_url, jar_path, false, helper);
		} catch (FileValidateException& err) {
			ERR(err.what());
			exit(1);
		}
	}

	void ClientInstance::ensure_instance_dir() {
		Instance::ensure_instance_dir();
		const fs::path mc_dir = dir / ".minecraft";
		create_dir_if_not_exists(mc_dir);
		create_dir_if_not_exists(mc_dir / "assets");
	}

	void ClientInstance::launch(User* user, const CachedPaths& paths) {
		mcvm::GameRunner game(
			parent->get_version(),
			dir / ".minecraft",
			dir / "client.jar",
			user,
			classpath,
			java->jre_path(paths)
		);
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

		const std::string major_version = std::to_string(json_access(
			json_access(version_json, "javaVersion"), "majorVersion"
		).GetUint());
		java = new AdoptiumJava(
			major_version
		);

		java->ensure_installed(paths);

		try {
			// Get the server jar
			const fs::path jar_path = server_dir / "server.jar";
			json::GenericObject server_download = json_access(
				json_access(version_json, "downloads"),
				"server"
			).GetObject();

			if (verbose) OUT_LIT("\tDownloading server jar...");
			download_cached_file(
				json_access(server_download, "url").GetString(),
				jar_path, false, helper
			);
		} catch (FileValidateException& err) {
			ERR(err.what());
			exit(1);
		}
		// Create the EULA
		write_file(server_dir / "eula.txt", "eula = true\n");
	}

	void ServerInstance::ensure_instance_dir() {
		Instance::ensure_instance_dir();
		create_dir_if_not_exists(dir / "server");
	}

	void ServerInstance::launch(UNUSED User* user, const CachedPaths& paths) {
		const fs::path server_path = dir / "server";
		const fs::path server_jar_path = server_path / "server.jar";
		assert(java != nullptr);
		const std::string java_command = java->jre_path(paths);
		const std::string command = java_command + " -jar " + server_jar_path.c_str();
		chdir(server_path.c_str());
		exit(system(command.c_str()));
	}
};
