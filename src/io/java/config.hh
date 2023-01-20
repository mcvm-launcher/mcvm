#pragma once
#include "lib/json.hh"
#include "net/net.hh"

namespace mcvm {
	// The set of all options pertaining to the java installation
	class JavaInstallation {
		public:
		JavaInstallation() = default;
		JavaInstallation(std::string _major_version);

		// The major Java version (e.g. 8 or 17)
		std::string major_version;
		virtual void ensure_installed(UNUSED const CachedPaths& paths) {}
		virtual std::string jre_path(UNUSED const CachedPaths& paths) {
			ASSERT_NOREACH();
		}

		virtual ~JavaInstallation() = default;
	};

	class AdoptiumJava : public JavaInstallation {
		public:
		using JavaInstallation::JavaInstallation;
		
		void ensure_installed(const CachedPaths& paths) override;
		std::string jre_path(const CachedPaths& paths) override;
	};
};
