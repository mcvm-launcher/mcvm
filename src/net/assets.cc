#include "net.hh"

namespace json = rapidjson;

namespace mcvm {
	CurlResult* update_assets() {
		std::cout << "Updating assets index..." << "\n";
		CURL* handle = curl_easy_init();

		// Download version manifest
		CurlResult* res = new CurlResult{};

		const std::filesystem::path manifest_file_path = get_mcvm_dir() / std::filesystem::path(ASSETS_DIR) / std::filesystem::path("version_manifest.json");
		curl_easy_setopt(handle, CURLOPT_URL, VERSION_MANIFEST_URL);
		curl_easy_setopt(handle, CURLOPT_WRITEFUNCTION, &write_data_to_file_and_str);

		create_leading_directories(manifest_file_path);
		res->file = fopen(manifest_file_path.c_str(), "wb");
		if (res->file) {
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

		const std::string test_json =
			"{"
				"\"versions\": ["
					"{"
						"\"id\": \"1.19.3\""
						"\"type\": \"release\""
						"\"url\": \"https://launchermeta.mojang.com/v1/packages/598eedd6f67db4aefbae6ed119029e3d7373ecf5/1.3.2.json\""
						"\"time\": \"2022-03-10T09:51:38+00:00\""
						"\"releaseTime\": \"2012-08-15T22:00:00+00:00\""
						"\"sha1\": \"598eedd6f67db4aefbae6ed119029e3d7373ecf5\""
						"\"complianceLevel\": 0"
					"}"
				"]"
			"}";

		if (manifest_file != nullptr) {
			json::Document doc;
			doc.Parse(manifest_file->str.c_str());

			std::string ver_url;
			std::string ver_hash;
			// We have to search them as they aren't indexed
			assert(doc.HasMember("versions"));
			for (auto&ver : doc["versions"].GetArray()) {
				json::GenericObject ver_obj = ver.GetObject();
				assert(ver_obj.HasMember("id"));
				if (ver_obj["id"].GetString() == version) {
					assert(ver_obj.HasMember("url"));
					assert(ver_obj.HasMember("sha1"));

					ver_url = ver_obj["url"].GetString();
					ver_hash = ver_obj["sha1"].GetString();
					assert(ver_hash.size() == 40);

					break;
				}
			}
			if (ver_url.empty()) {
				delete manifest_file;
				throw VersionNotFoundException();
			}
		}
		
		delete manifest_file;
	}
};
