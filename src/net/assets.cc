#include "net.hh"

namespace mcvm {
	void update_assets() {
		std::cout << "Updating assets index..." << "\n";
		CURL* handle = curl_easy_init();
		FILE* manifest_file;
		// Download version manifest
		curl_easy_setopt(handle, CURLOPT_URL, "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json");
		curl_easy_setopt(handle, CURLOPT_WRITEFUNCTION, &write_data_to_file);
    curl_easy_setopt(handle, CURLOPT_WRITEDATA, manifest_file);
		manifest_file = fopen(PATH_CONCAT_2(ASSETS_DIR, ), "wb");
		if (manifest_file) {
			const CURLcode success = curl_easy_perform(handle);
		}
	}
};
