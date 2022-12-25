#include "net.hh"

namespace json = rapidjson;

namespace mcvm {
	std::string update_assets() {
		// Ensure assets dir
		const fs::path assets_path = get_mcvm_dir() / fs::path(ASSETS_DIR);
		create_dir_if_not_exists(assets_path);

		OUT("Updating assets index...");

		// Download version manifest
		const fs::path manifest_file_path = assets_path / fs::path("version_manifest.json");
		DownloadHelper helper(DownloadHelper::FILE_AND_STR, VERSION_MANIFEST_URL, manifest_file_path);
		bool success = helper.perform();
		return helper.get_str();
	}

	void obtain_libraries(const std::string& version) {
		const std::string manifest_file = update_assets();

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

		json::Document doc;
		doc.Parse(manifest_file.c_str());

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
			throw VersionNotFoundException();
		}
		// We now have to download the libraries manifest
		// TODO: Checksum

		const std::string index_file_name = version + ".json";
		const fs::path index_file_path = get_mcvm_dir() / fs::path(ASSETS_DIR) / fs::path(index_file_name);
		DownloadHelper helper(DownloadHelper::FILE_AND_STR, ver_url, index_file_path);
		bool success = helper.perform();
	}
};
