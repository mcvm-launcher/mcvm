#pragma once
#include "data/resource.hh"
#include "net/net.hh"

namespace mcvm {
	// The result from evaluation
	struct PkgEvalResult {
		std::string pkg_name;
		std::string pkg_version;
	};

	class PkgAST;
	class ParseData;

	struct EvalType {
		// The extent of evaluation to perform on a package
		enum __EvalType {
			INSTALL, // Evaluates the full package while actually running things
			GET_INFO // Evaluates the full package without actually running anything
		};
	};

	// The level of evaluation to be performed
	enum RunLevel {
		ALL, // Run all commands
		RESTRICTED, // Restrict the scope of commands
		INFO, // Only run commands that set information
		NONE // Don't run any commands
	};

	// A mcvm package
	class Package {
		protected:

		const std::string name;
		fs::path location;
		// Contents of the package build script
		std::string contents;

		Package(const std::string& _name, const fs::path& _location);

		public:
		// Ensure that the package contents are stored in memory
		virtual void ensure_contents() {}
		// Parse the package contents
		PkgAST* parse();
		void evaluate(ParseData* ret, const std::string& routine, RunLevel level) {}

		virtual ~Package() = default;
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
