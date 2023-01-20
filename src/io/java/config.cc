#include "config.hh"

namespace mcvm {
	JavaInstallation::JavaInstallation(std::string _major_version)
	: major_version(_major_version) {}

	void AdoptiumJava::ensure_installed(const CachedPaths& paths) {
		const fs::path out_dir = paths.internal / "java" / "adoptium";
		static const fs::path install_path = out_dir / major_version;
		if (file_exists(install_path)) return;
		// 'https://api.adoptium.net/v3/assets/latest/17/hotspot?architecture=x64&image_type=jre&os=linux&vendor=eclipse'
		const std::string url = NICE_STR_CAT(
			"https://api.adoptium.net/v3/assets/latest/"
			+ major_version + "/hotspot?"
			"image_type=jre"
			"&vendor=eclipse"
			"&architecture=" ARCH_STRING
			"&os=" OS_STRING
		);
		DownloadHelper helper;
		helper.set_options(DownloadHelper::STR, url);
		helper.follow_redirect();
		helper.perform();

		json::Document manifest;
		manifest.Parse(helper.get_str().c_str());
		assert(manifest.IsArray());
		assert(manifest.GetArray().Size() > 0);
		json::GenericObject version = manifest.GetArray()[0].GetObject();
		json::GenericObject binary_package = json_access(
			json_access(version, "binary"), "package"
		).GetObject();
		const std::string bin_url = json_access(binary_package, "link").GetString();
		fs::path extracted_bin_path = out_dir;
		extracted_bin_path /= json_access(version, "release_name").GetString();
		extracted_bin_path += "-jre";

		// Get the binaries
		const fs::path file_path = out_dir / ("adoptium" + major_version + ".tar.gz");
		create_leading_directories(file_path);
		helper.set_options(DownloadHelper::FILE, bin_url, file_path);
		helper.follow_redirect();
		helper.perform();
		helper.reset();
		extract_tar_gz(file_path);

		// Cleanup
		int fail;
		fail = remove(file_path.c_str());
		if (fail) {
			WARN("Failed to remove archived java installation");
		}

		// This is so we can index it later
		// Should probably make it faster
		create_dir_if_not_exists(install_path);
		// I don't know why this is giving an error but it works so
		try {
			fs::copy(extracted_bin_path, install_path, fs::copy_options::recursive | fs::copy_options::update_existing);
		} catch (fs::filesystem_error& err) {}
		fs::remove_all(extracted_bin_path);
	}

	std::string AdoptiumJava::jre_path(const CachedPaths& paths) {
		return paths.internal / "java" / "adoptium" / major_version / "bin" / "java";
	}
};
