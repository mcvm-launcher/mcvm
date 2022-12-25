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
		create_impl();
	}

	void Instance::ensure_instance_dir() {
		create_leading_directories(dir);
	}

	ClientInstance::ClientInstance(Profile* _parent, const std::string _name, const fs::path& mcvm_dir)
	: Instance(_parent, _name, mcvm_dir, CLIENT_INSTANCES_DIR) {}

	ServerInstance::ServerInstance(Profile* _parent, const std::string _name, const fs::path& mcvm_dir)
	: Instance(_parent, _name, mcvm_dir, SERVER_INSTANCES_DIR) {}
};
