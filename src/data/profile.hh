#pragma once
#include "package/package.hh"
#include "io/game.hh"

namespace mcvm {
	class Instance;

	// A profile, which holds game settings and can be depended on by runnable instances 
	class Profile {
		const std::string name;
		MinecraftVersion version;
		std::vector<Package*> packages;

		public:
		std::map<std::string, Instance*> instances;

		Profile(const std::string _name, MinecraftVersion _version);

		const std::string& get_name();
		MinecraftVersion get_version();
		void add_package(Package* pkg);
		void update_packages();
		void delete_all_packages();
		void create_instances(const CachedPaths& paths);

		~Profile() = default;
	};

	// Base for instance
	class Instance {
		protected:
		Instance(Profile* _parent, const std::string _name, const CachedPaths& paths, const std::string& subpath);

		// The profile that this instance is created from
		Profile* parent = nullptr;

		// Version json downloaded from Mojang
		json::Document version_json;

		public:
		const std::string name;
		fs::path dir;

		// Make sure that the instance has a cached rendered config
		// void ensure_cached() {}

		// Create the instance and all of its files
		virtual void create(UNUSED const CachedPaths& paths, UNUSED bool verbose = true);

		// Make sure that the instance has a created directory
		virtual void ensure_instance_dir();

		// Run the instance
		virtual void launch(UNUSED User* user, UNUSED const CachedPaths& paths) {}

		// Obtain the version of the instance
		MinecraftVersion get_version();

		virtual ~Instance() = default;
	};

	// A profile that also holds client-specific resources
	class ClientInstance : public Instance {
		// Resources
		std::vector<WorldResource*> worlds;

		public:
		ClientInstance(Profile* _parent, const std::string _name, const CachedPaths& paths);

		void create(const CachedPaths& paths, bool verbose = true) override;
		void ensure_instance_dir() override;
		void launch(User* user, const CachedPaths& paths) override;
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
		ServerInstance(Profile* _parent, const std::string _name, const CachedPaths& paths);

		void create(const CachedPaths& paths, bool verbose = true) override;
		void ensure_instance_dir() override;
	};
};
