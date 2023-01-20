#include "config.hh"

namespace mcvm {
	JavaInstallation::JavaInstallation(std::string _major_version)
	: major_version(_major_version) {}

	void AdoptiumJava::ensure_installed(const CachedPaths& paths) {
		const fs::path out_dir = paths.internal / "java" / "adoptium";
		const fs::path file_path = out_dir / ("adoptium" + major_version + ".tar.gz");
		create_leading_directories(file_path);
		const std::string url = NICE_STR_CAT(
			"https://api.adoptium.net/v3/binary/latest/"
			+ major_version +
			"/ga/"
			OS_STRING
			"/"
			ARCH_STRING
			"/jre/hotspot/normal/eclipse"
		);

		DownloadHelper helper;
		helper.set_options(DownloadHelper::FILE, url, file_path);
		helper.follow_redirect();
		helper.perform();
		helper.reset();
		extract_tar_gz(file_path);
		remove(file_path.c_str());
	}
};
