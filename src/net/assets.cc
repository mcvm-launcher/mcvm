#include "net.hh"\

namespace mcvm {
	void update_assets() {
		std::cout << "Updating assets index..." << "\n";
		CURL* handle = curl_easy_init();
		FILE* manifest_file;
		//const std::string manifest_file_path = path_concat(path_concat(get_mcvm_dir(), ASSETS_DIR), "version_manifest.json");
		const std::string manifest_file_path = "version_manifest.json";
		// Download version manifest
		curl_easy_setopt(handle, CURLOPT_URL, "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json");\
		//curl_easy_setopt(handle, CURLOPT_VERBOSE, 1L);
		curl_easy_setopt(handle, CURLOPT_WRITEFUNCTION, &write_data_to_file);
		std::cout << manifest_file_path << "\n";
		manifest_file = fopen(manifest_file_path.c_str(), "wb");
		if (manifest_file) {
			curl_easy_setopt(handle, CURLOPT_WRITEDATA, manifest_file);
			curl_easy_perform(handle);
			fclose(manifest_file);
		}
		curl_easy_cleanup(handle);
	}
};
