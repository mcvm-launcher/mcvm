#pragma once
#include <string>

namespace mcvm {
	typedef std::string MCVersion;
	typedef std::string ResourceVersion;

	// Type of modloader
	enum ModType {
		FABRIC,
		FORGE,
		QUILT
	};

	struct GlobalResources;
	// Global settings struct
	struct GlobalSettings {
		GlobalResources* resources;
	};
};
