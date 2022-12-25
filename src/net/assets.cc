#include "net.hh"

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

	void obtain_version_json(const std::string& version, json::Document* ret) {
		OUT_LIT("Downloading version json...");
		const std::string manifest_file = update_assets();

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
		// We now have to download the manifest for the specific version
		// TODO: Checksum

		const std::string index_file_name = version + ".json";
		const fs::path index_file_path = get_mcvm_dir() / fs::path(ASSETS_DIR) / fs::path(index_file_name);
		DownloadHelper helper(DownloadHelper::FILE_AND_STR, ver_url, index_file_path);
		bool success = helper.perform();
		ret->Parse(helper.get_str().c_str());
	}

	void obtain_libraries(const std::string& version, json::Document* ret) {
		obtain_version_json(version, ret);

		OUT_LIT("Downloading libraries...");
		assert(ret->HasMember("libraries"));
	}
};
