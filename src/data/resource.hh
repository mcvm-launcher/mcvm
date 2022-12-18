#pragma once
#include <string>

namespace mcvm {
	typedef std::string MCVersion;

	class Resource {
		public:
		Resource(const MCVersion _version) : version(_version) {}

		MCVersion version;
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
	};

	
};
