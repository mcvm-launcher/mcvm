#pragma once
#include "lib/json.hh"
#include "net/net.hh"

namespace mcvm {
	// The set of all options pertaining to the java installation
	class JavaInstallation {
		protected:
		// The major Java version (e.g. 8 or 17)
		const std::string major_version;

		public:
		JavaInstallation(std::string _major_version);

		virtual void ensure_installed(const CachedPaths& paths) {}
	};

	class AdoptiumJava : public JavaInstallation {
		public:
		using JavaInstallation::JavaInstallation;
		
		void ensure_installed(const CachedPaths& paths) override;
	};
};
