#pragma once
#include <string>
#include <exception>

namespace mcvm {
	typedef std::string MCVersion;
	typedef std::string ResourceVersion;

	// Enum of different types of modloaders
	enum ModType {
		FABRIC,
		FORGE,
		QUILT
	};

	// Enums for different types / subdivisions of Minecraft versions
	enum VersionType {
		RELEASE,
		SNAPSHOT,
		OLD_ALPHA
	};

	// Thrown when a Minecraft version does not exist
	struct VersionNotFoundException : public std::exception {
		const char* what() {
			return "Minecraft version does not exist";
		}
	};

	struct GlobalResources;
	// Global settings struct
	struct GlobalSettings {
		GlobalResources* resources;
	};
};
