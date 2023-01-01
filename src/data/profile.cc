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

	ClientInstance::ClientInstance(Profile* _parent, const std::string _name, const fs::path& mcvm_dir)
	: Instance(_parent, _name, mcvm_dir, CLIENT_INSTANCES_DIR) {}

	void ClientInstance::create() {
		ensure_instance_dir();

		json::Document doc;
		obtain_libraries(parent->get_version(), &doc);

		// Get the client jar
		const fs::path jar_path = dir / "client.jar";
		assert(doc.HasMember("downloads"));
		const json::GenericObject client_download = doc["downloads"]["client"].GetObject();
		DownloadHelper helper;
		helper.set_options(DownloadHelper::FILE, client_download["url"].GetString(), jar_path);
		OUT_LIT("Downloading client jar...");
		helper.perform();
	}

	void ClientInstance::ensure_instance_dir() {
		Instance::ensure_instance_dir();
		create_dir_if_not_exists(dir / ".minecraft");
	}

	ServerInstance::ServerInstance(Profile* _parent, const std::string _name, const fs::path& mcvm_dir)
	: Instance(_parent, _name, mcvm_dir, SERVER_INSTANCES_DIR), server_dir(dir / "server") {}

	void ServerInstance::create() {
		ensure_instance_dir();
		
		json::Document doc;
		obtain_version_json(parent->get_version(), &doc);

		// Get the server jar
		const fs::path jar_path = server_dir / "server.jar";
		assert(doc.HasMember("downloads"));
		const json::GenericObject client_download = doc["downloads"]["server"].GetObject();
		DownloadHelper helper;
		helper.set_options(DownloadHelper::FILE, client_download["url"].GetString(), jar_path);
		OUT_LIT("Downloading server jar...");
		helper.perform();

		// Create the EULA
		write_file(server_dir / "eula.txt", "eula = true\n");
	}

	void ServerInstance::ensure_instance_dir() {
		Instance::ensure_instance_dir();
		create_dir_if_not_exists(dir / "server");
	}
};
