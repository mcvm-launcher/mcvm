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

	// An abstraction of a resource that allows for packages and the such
	// The template is only for type checking reasons
	template <typename T>
	class ResourceRef {};

	template <typename T>
	class StaticResourceRef : public ResourceRef<T> {
		Resource* res;
	};

	// typedef std::vector<ResourceRef<T>&> ResourceList
	
	// Global shared resources
	struct GlobalResources {
		std::vector<ResourceRef<WorldResource>*> worlds;
		std::vector<ResourceRef<ResourcePackResource>*> resource_packs;
		std::vector<ResourceRef<DatapackResource>*> datapacks;
		std::vector<ResourceRef<PluginResource>*> plugins;
	};
};
