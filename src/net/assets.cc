#include "net.hh"

namespace mcvm {
	CurlResult* update_assets() {
		std::cout << "Updating assets index..." << "\n";
		CURL* handle = curl_easy_init();

		// Download version manifest
		FILE* manifest_file;
		char* manifest_file_str;
		CurlResult* res = new CurlResult{ manifest_file, manifest_file_str };

		const std::filesystem::path manifest_file_path = get_mcvm_dir() / std::filesystem::path(ASSETS_DIR) / std::filesystem::path("version_manifest.json");
		curl_easy_setopt(handle, CURLOPT_URL, VERSION_MANIFEST_URL);
		curl_easy_setopt(handle, CURLOPT_WRITEFUNCTION, &write_data_to_file_and_str);

		create_leading_directories(manifest_file_path);
		manifest_file = fopen(manifest_file_path.c_str(), "wb");
		if (manifest_file) {
			// We gotta update this again since it changed from fopen
			res->file = manifest_file;
			curl_easy_setopt(handle, CURLOPT_WRITEDATA, res);
			curl_easy_perform(handle);
		} else {
			return nullptr;
		}
		curl_easy_cleanup(handle);

		return res;
	}

	void obtain_libraries(const std::string& version) {
		CurlResult* manifest_file = update_assets();

		if (manifest_file != nullptr) {
			rapidjson::Document doc;
			//doc.Parse(manifest_file->str);
			std::cout << manifest_file->str << "\n";
		}
		
		delete manifest_file;
	}
};
