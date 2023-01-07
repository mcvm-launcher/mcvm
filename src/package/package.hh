#pragma once
#include "data/resource.hh"
#include "net/net.hh"

#include <map>

namespace mcvm {
	class PkgInstruction;
	struct PkgEvalData;
	struct PkgEvalGlobals;

	class PkgBlock {
		public:
		PkgBlock() = default;

		std::vector<PkgInstruction*> instructions;
		PkgBlock* parent = nullptr;

		void evaluate(PkgEvalData& data, const PkgEvalGlobals& global);
	};

	// Used to download resources at the end
	class ResourceAquirer {
		public:
		ResourceAquirer();
	};

	// The level of evaluation to be performed
	enum RunLevel {
		ALL, // Run all commands
		RESTRICTED, // Restrict the scope of commands
		INFO, // Only run commands that set information
		NONE // Don't run any commands
	};
	
	// Package eval global information
	struct PkgEvalGlobals {
		RunLevel level = RunLevel::ALL;
		fs::path working_directory;
		std::string package_requested_version;
		MCVersion mc_version;
		ModType modloader = ModType::FABRIC; 
		MinecraftSide side = MinecraftSide::CLIENT;
	};

	// The result from evaluation
	struct PkgEvalData {
		std::string pkg_name;
		std::string pkg_version;
		// TODO: Temporary
		// The list of resources to be aquired once the whole package is evaluated
		std::vector<ResourceAquirer*> resources;

		~PkgEvalData() {
			DEL_VECTOR(resources);
		}
	};

	class PkgAST {
		public:
		std::map<std::string, PkgBlock> routines;

		PkgAST() = default;

		~PkgAST() {
			for (std::map<std::string, PkgBlock>::iterator i = routines.begin(); i != routines.end(); i++) {
				PkgBlock rtn = i->second;
				DEL_VECTOR(rtn.instructions);
			}
		}
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
		void evaluate(PkgEvalData& ret, const std::string& routine_name, const PkgEvalGlobals& global);

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
