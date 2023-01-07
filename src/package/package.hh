#pragma once
#include "data/resource.hh"
#include "net/net.hh"

namespace mcvm {
	// Used to download resources at the end
	class ResourceAquirer {
		public:
		ResourceAquirer();
	};

	// The result from evaluation
	struct PkgEvalData {
		std::string pkg_name;
		std::string pkg_version;
		std::string package_requested_version;
		MCVersion mc_version;
		// TODO: Temporary
		ModType modloader = ModType::FABRIC; 
		MinecraftSide side = MinecraftSide::CLIENT;
		// The list of resources to be aquired once the whole package is evaluated
		std::vector<ResourceAquirer*> resources;

		~PkgEvalData() {
			for (unsigned int i = 0; i < resources.size(); i++) {
				delete resources[i];
			}
		}
	};

	class PkgAST;

	// The level of evaluation to be performed
	enum RunLevel {
		ALL, // Run all commands
		RESTRICTED, // Restrict the scope of commands
		INFO, // Only run commands that set information
		NONE // Don't run any commands
	};

	// Package eval global information
	struct PkgEvalGlobals {
		RunLevel level;
		const fs::path& working_directory;
	};

	// A mcvm package
	class Package {
		protected:

		const std::string name;
		fs::path location;
		// Contents of the package build script
		std::string contents;
		// The abstract syntax tree
		PkgAST* ast = nullptr;

		Package(const std::string& _name, const fs::path& _location);

		public:
		// Ensure that the package contents are stored in memory
		virtual void ensure_contents() {}
		// Parse the package contents
		void parse();
		void evaluate(PkgEvalData& ret, const std::string& routine_name, RunLevel level);

		virtual ~Package();
	};

	// A package installed from the internet, which has more restrictions
	class RemotePackage : public Package {
		const std::string url;

		public:
		RemotePackage(const std::string& _name, const std::string& _url, const fs::path& cache_dir);

		void ensure_contents() override;
	};

	// A package installed from the local filesystem
	class LocalPackage : public Package {
		public:
		LocalPackage(const std::string& _name, const fs::path& path);

		void ensure_contents() override;
	};
};
