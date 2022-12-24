#pragma once
#include "resource.hh"

namespace mcvm {
	// A profile, which holds game settings and can be depended on by runnable instances 
	class Profile {
		Profile() = default;
	};

	// Base for profile
	class Instance {
		// The profile that this instance is created from
		Profile* parent = nullptr;

		public:
		Instance(Profile* _parent, MCVersion& _version);
		MCVersion version;

		// Make sure that the profile has a cached rendered config
		void ensure_cached() {}
	};

	// A profile that also holds client-specific resources
	class ClientInstance : public Instance {
		// Resources
		// Important to remember that this is only a list of worlds installed by packages and managed by mcvm
		std::vector<WorldResource*> worlds;

		public:
		using Instance::Instance;
	};

	class ServerInstance : public Instance {
		// Resources
		std::vector<PluginResource*> plugins;
		// A server can only have one world but we store multiple as well for
		// easy switching and bungeecord/multiverse and stuff
		std::vector<WorldResource*> worlds;
		WorldResource* current_world;

		public:
		using Instance::Instance;
	};
};
