#pragma once
#include "resource.hh"
#include "package/package.hh"

namespace mcvm {
	// A profile, which holds game settings and can be depended on by runnable instances 
	class Profile {
		const std::string name;
		MCVersion version;
		std::vector<Package*> packages;

		public:
		Profile(const std::string _name, MCVersion _version);

		MCVersion get_version();
		void add_package(Package* pkg);
		void delete_all_packages();
	};

	// Base for instance
	class Instance {
		protected:
		Instance(Profile* _parent, const std::string _name, const fs::path& mcvm_dir, const std::string& subpath);

		// The profile that this instance is created from
		Profile* parent = nullptr;

		public:
		const std::string name;
		fs::path dir;

		// Make sure that the instance has a cached rendered config
		// void ensure_cached() {}

		// Create the instance and all of its files
		virtual void create();

		// Make sure that the instance has a created directory
		virtual void ensure_instance_dir();
	};

	// A profile that also holds client-specific resources
	class ClientInstance : public Instance {
		// Resources
		std::vector<WorldResource*> worlds;

		public:
		ClientInstance(Profile* _parent, const std::string _name, const fs::path& mcvm_dir);

		void create() override;
		void ensure_instance_dir() override;
	};

	class ServerInstance : public Instance {
		// Resources
		std::vector<PluginResource*> plugins;
		// A server can only have one world but we store multiple as well for
		// easy switching and bungeecord/multiverse and stuff
		std::vector<WorldResource*> worlds;
		WorldResource* current_world;

		const fs::path server_dir;

		public:
		ServerInstance(Profile* _parent, const std::string _name, const fs::path& mcvm_dir);

		void create() override;
		void ensure_instance_dir() override;
	};
};
