#pragma once
#include "resource.hh"

namespace mcvm {
	// A profile, which holds game settings and can be depended on by runnable instances 
	class Profile {
		const std::string name;
		MCVersion version;

		public:
		Profile(const std::string _name, MCVersion _version);

		MCVersion get_version();
	};

	// Base for profile
	class Instance {
		// The profile that this instance is created from
		Profile* parent = nullptr;

		protected:
		// Implementation of create
		virtual void create_impl() {}

		Instance(Profile* _parent, const std::string _name, const fs::path& mcvm_dir, const std::string& subpath);

		public:
		const std::string name;
		fs::path dir;

		// Make sure that the instance has a cached rendered config
		// void ensure_cached() {}

		// Create the instance and all of its files
		void create();

		// Make sure that the instance has a created directory
		void ensure_instance_dir();
	};

	// A profile that also holds client-specific resources
	class ClientInstance : public Instance {
		// Resources
		std::vector<WorldResource*> worlds;

		protected:
		void create_impl() override {}

		public:
		ClientInstance(Profile* _parent, const std::string _name, const fs::path& mcvm_dir);
	};

	class ServerInstance : public Instance {
		// Resources
		std::vector<PluginResource*> plugins;
		// A server can only have one world but we store multiple as well for
		// easy switching and bungeecord/multiverse and stuff
		std::vector<WorldResource*> worlds;
		WorldResource* current_world;

		protected:
		void create_impl() override {}

		public:
		ServerInstance(Profile* _parent, const std::string _name, const fs::path& mcvm_dir);
	};
};
