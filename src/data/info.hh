#pragma once
#include <string>
#include <exception>

namespace mcvm {
	typedef std::string MCVersionString;
	typedef std::string ResourceVersion;

	// Enum of different types of modloaders
	enum ModType {
		VANILLA,
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

	// Enum for the sidedness
	enum MinecraftSide {
		CLIENT,
		SERVER
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
