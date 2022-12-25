#include "package.hh"

namespace mcvm {
	Package::Package(const std::string &_name, const fs::path& _location)
	: name(_name), location(_location) {}

	RemotePackage::RemotePackage(const std::string& name, const std::string& _url, const fs::path& cache_dir)
	: Package(name, cache_dir / CACHED_PACKAGES_DIR / add_package_extension(name)), url(_url) {}

	void RemotePackage::ensure_contents(const fs::path cache_dir) {
		const fs::path cached_package_path = cache_dir / CACHED_PACKAGES_DIR / add_package_extension(name);
		// Check if it is already downloaded
		if (!file_exists(cached_package_path)) {
			DownloadHelper helper(DownloadHelper::FILE_AND_STR, url, cached_package_path);
			bool success = helper.perform();
			contents = helper.get_str();
		}

		read_file(cached_package_path, contents);
	}

	LocalPackage::LocalPackage(const std::string& _name, const fs::path& path)
	: Package(_name, path) {}

	void LocalPackage::ensure_contents(const fs::path cache_dir) {
		read_file(location, contents);
	}
};
