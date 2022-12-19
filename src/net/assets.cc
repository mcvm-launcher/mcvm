#include "net.hh"

namespace mcvm {
	void update_assets() {
		std::cout << "Updating assets index..." << "\n";
		CURL* handle = curl_easy_init();

		// Download version manifest
		FILE* manifest_file;
		const std::filesystem::path mfp = get_mcvm_dir() / std::filesystem::path(ASSETS_DIR) / std::filesystem::path("version_manifest.json");
		curl_easy_setopt(handle, CURLOPT_URL, "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json");
		curl_easy_setopt(handle, CURLOPT_WRITEFUNCTION, &write_data_to_file);

		create_leading_directories(mfp);
		manifest_file = fopen(mfp.c_str(), "wb");
		if (manifest_file) {
			curl_easy_setopt(handle, CURLOPT_WRITEDATA, manifest_file);
			curl_easy_perform(handle);
			fclose(manifest_file);
		}
		curl_easy_cleanup(handle);
	}
};
