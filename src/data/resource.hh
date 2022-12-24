#pragma once
#include "info.hh"

#include <vector>

namespace mcvm {
		// Object for a file installed in your Minecraft directory
		class Resource {
		public:
		Resource(const MCVersion _mc_vers, const ResourceVersion _vers)
		: mc_vers(_mc_vers), vers(_vers) {}

		// Ensures that a resource is available for use by Minecraft
		virtual void ensure_render() {}

		MCVersion mc_vers;
		ResourceVersion vers;
	};

	class ResourcePackResource : public Resource {
		public:
		using Resource::Resource;
	};

	class DatapackResource : public Resource {
		public:
		using Resource::Resource;
	};

	class WorldResource : public Resource {
		public:
		using Resource::Resource;

		std::vector<DatapackResource> datapacks = {};
	};

	class ModResource : public Resource {
		public:
		ModResource(const MCVersion _mc_vers, const ResourceVersion _vers, const ModType _type)
		: Resource(_mc_vers, _vers), type(_type) {}

		const ModType type;
	};

	// A Bukkit plugin
	class PluginResource : public Resource {
		public:
		using Resource::Resource;
	};

	// An managed pointer to a resource that allows for packages and the such
	// Not used now, maybe not ever
	template <typename Resource_T>
	class ResourceRef {
		public:
		enum ResourceRefType {

		};
	};

	// typedef std::vector<ResourceRef<T>&> ResourceList
	
	// Global shared resources
	struct GlobalResources {
		std::vector<WorldResource*> worlds;
		std::vector<ResourcePackResource*> resource_packs;
		std::vector<DatapackResource*> datapacks;
		std::vector<PluginResource*> plugins;
	};
};
