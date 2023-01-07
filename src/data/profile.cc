#include "profile.hh"

namespace mcvm {
	Profile::Profile(const std::string _name, MCVersion _version)
	: name(_name), version(_version) {}

	MCVersion Profile::get_version() {
		return version;
	}

	void Profile::add_package(Package* pkg) {
		packages.push_back(pkg);
	}

	void Profile::delete_all_packages() {
		for (std::vector<Package*>::iterator i = packages.begin(); i != packages.end(); i++) {
			delete *i;
		}
		packages = {};
	}

	Instance::Instance(Profile* _parent, const std::string _name, const fs::path& mcvm_dir, const std::string& subpath)
	: parent(_parent), name(_name), dir(mcvm_dir / subpath / name) {}

	void Instance::create() {
		ensure_instance_dir();
	}

	void Instance::ensure_instance_dir() {
		create_leading_directories(dir);
		create_dir_if_not_exists(dir);
	}
	
	MCVersion Instance::get_version() {
		return parent->get_version();
	}

	ClientInstance::ClientInstance(Profile* _parent, const std::string _name, const fs::path& mcvm_dir)
	: Instance(_parent, _name, mcvm_dir, CLIENT_INSTANCES_DIR) {}

	void ClientInstance::create() {
		ensure_instance_dir();
		const fs::path mc_dir = dir / ".minecraft";

		std::shared_ptr<DownloadHelper> helper = obtain_libraries(parent->get_version(), &version_json);

		// Get the client jar
		json::GenericObject client_download = json_access(json_access(version_json, "downloads"), "client").GetObject();
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

	void ClientInstance::launch(User* user) {
		mcvm::GameRunner game(parent->get_version(), dir / ".minecraft", dir / "client.jar", user);
		game.parse_args(&version_json);
		game.launch();
	}

	ServerInstance::ServerInstance(Profile* _parent, const std::string _name, const fs::path& mcvm_dir)
	: Instance(_parent, _name, mcvm_dir, SERVER_INSTANCES_DIR), server_dir(dir / "server") {}

	void ServerInstance::create() {
		ensure_instance_dir();
		
		std::shared_ptr<DownloadHelper> helper = obtain_version_json(parent->get_version(), &version_json);

		// Get the server jar
		const fs::path jar_path = server_dir / "server.jar";
		json::GenericObject server_download = json_access(json_access(version_json, "downloads"), "server").GetObject();
		download_cached_file(json_access(server_download, "url").GetString(), jar_path, false, helper);

		// Create the EULA
		write_file(server_dir / "eula.txt", "eula = true\n");
	}

	void ServerInstance::ensure_instance_dir() {
		Instance::ensure_instance_dir();
		create_dir_if_not_exists(dir / "server");
	}
};
