#include "package.hh"

namespace mcvm {
	Package::Package(const std::string &_name, const fs::path& _location)
	: name(_name), location(_location) {}

	RemotePackage::RemotePackage(const std::string& name, const std::string& _url, const fs::path& cache_dir)
	: Package(name, cache_dir / CACHED_PACKAGES_DIR / add_package_extension(name)), url(_url) {}

	void RemotePackage::ensure_contents() {
		// Check if it is already downloaded
		if (!file_exists(location)) {
			create_leading_directories(location);
			DownloadHelper helper;
			helper.set_options(DownloadHelper::FILE_AND_STR, url, location);
			helper.perform();
			contents = helper.get_str();
			return;
		}

		read_file(location, contents);
	}

	LocalPackage::LocalPackage(const std::string& _name, const fs::path& path)
	: Package(_name, path) {}

	void LocalPackage::ensure_contents() {
		read_file(location, contents);
	}
};
