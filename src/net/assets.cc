#include "net.hh"

namespace mcvm {
	std::shared_ptr<DownloadHelper> update_assets() {
		// Ensure assets dir
		const fs::path assets_path = get_mcvm_dir() / fs::path(ASSETS_DIR);
		create_dir_if_not_exists(assets_path);

		OUT("Updating assets index...");

		// Download version manifest
		const fs::path manifest_file_path = assets_path / fs::path("version_manifest.json");
		std::shared_ptr<DownloadHelper> helper = std::make_shared<DownloadHelper>();
		helper->set_options(DownloadHelper::FILE_AND_STR, VERSION_MANIFEST_URL, manifest_file_path);
		helper->perform();
		return helper;
	}

	void obtain_version_json(const std::string& version, json::Document* ret) {
		OUT_LIT("Downloading version json...");
		std::shared_ptr<DownloadHelper> helper = update_assets();
		const std::string manifest_file = helper->get_str();

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
		const std::string index_file_name = version + ".json";
		const fs::path index_file_path = get_mcvm_dir() / fs::path(ASSETS_DIR) / fs::path(index_file_name);
		helper->set_options(DownloadHelper::FILE_AND_STR, ver_url, index_file_path);
		helper->perform();
		helper->sha1_checksum(ver_hash);
		ret->Parse(helper->get_str().c_str());
	}

	void obtain_libraries(const std::string& version, json::Document* ret) {
		obtain_version_json(version, ret);

		const fs::path libraries_path = get_mcvm_dir() / "libraries";
		create_dir_if_not_exists(libraries_path);

		OUT_LIT("Downloading libraries...");

		MultiDownloadHelper multi_helper;

		assert(ret->HasMember("libraries"));
		for (auto& lib_val : ret->operator[]("libraries").GetArray()) {
			const json::GenericObject lib = lib_val.GetObject();
			assert(lib.HasMember("downloads"));
			const json::GenericObject download_artifact = lib["downloads"]["artifact"].GetObject();

			assert(download_artifact.HasMember("path"));
			const char* path_str = download_artifact["path"].GetString();
			const fs::path path = libraries_path / path_str;
			// If we already have the library don't download it again
			if (file_exists(path)) continue;
			create_leading_directories(path);

			assert(download_artifact.HasMember("url"));
			const char* url = download_artifact["url"].GetString();

			assert(lib.HasMember("name"));
			const char* name = lib["name"].GetString();

			// Check rules
			if (lib.HasMember("rules")) {
				bool rule_fail = false;
				for (auto& rule : lib["rules"].GetArray()) {
					assert(rule.HasMember("action"));
					const std::string_view action = rule["action"].GetString();
					const std::string_view os_name = rule["os"]["name"].GetString();
					const std::string test = OS_STRING;
					if (
						(action == "allow" && os_name != OS_STRING) ||
						(action == "disallow" && os_name == OS_STRING)
					) {
						rule_fail = true;
					}
				}
				if (rule_fail) continue;
			}

			OUT("Downloading " << name);
			std::shared_ptr<DownloadHelper> helper = std::make_shared<DownloadHelper>();
			helper->set_options(DownloadHelper::FILE, url, path);
			multi_helper.add_helper(helper);
		}
		multi_helper.perform_blocking();
	}
};
