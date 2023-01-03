#include "net.hh"

namespace mcvm {
	// README: https://wiki.vg/Game_files

	std::shared_ptr<DownloadHelper> get_version_manifest() {
		// Ensure assets dir
		const fs::path assets_path = get_mcvm_dir() / fs::path(ASSETS_DIR);
		create_dir_if_not_exists(assets_path);

		OUT("Obtaining version index...");

		const fs::path manifest_file_path = assets_path / fs::path("version_manifest.json");
		std::shared_ptr<DownloadHelper> helper = std::make_shared<DownloadHelper>();
		helper->set_options(DownloadHelper::FILE_AND_STR, VERSION_MANIFEST_URL, manifest_file_path);
		helper->perform();
		return helper;
	}

	std::shared_ptr<DownloadHelper> obtain_version_json(const std::string& version, json::Document* ret) {
		OUT_LIT("Downloading version json...");
		std::shared_ptr<DownloadHelper> helper = get_version_manifest();
		const std::string manifest_file = helper->get_str();

		json::Document doc;
		doc.Parse(manifest_file.c_str());

		std::string ver_url;
		std::string ver_hash;
		// We have to search them as they aren't indexed
		for (auto&ver : json_access(doc, "versions").GetArray()) {
			json::GenericObject ver_obj = ver.GetObject();
			if (json_access(ver_obj, "id").GetString() == version) {
				ver_url = json_access(ver_obj, "url").GetString();
				ver_hash = json_access(ver_obj, "sha1").GetString();
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

		return helper;
	}

	void obtain_libraries(const std::string& version, json::Document* ret) {
		std::shared_ptr<DownloadHelper> helper = obtain_version_json(version, ret);

		const fs::path libraries_path = get_mcvm_dir() / "libraries";
		create_dir_if_not_exists(libraries_path);

		OUT_LIT("Downloading libraries...");

		MultiDownloadHelper multi_helper;

		for (auto& lib_val : json_access(ret, "libraries").GetArray()) {
			json::GenericObject lib = lib_val.GetObject();
			json::GenericObject download_artifact = json_access(json_access(lib, "downloads"), "artifact").GetObject();

			const char* path_str = json_access(download_artifact, "path").GetString();
			const fs::path path = libraries_path / path_str;
			// If we already have the library don't download it again
			if (file_exists(path)) continue;
			create_leading_directories(path);

			const char* url = json_access(download_artifact, "url").GetString();

			const char* name = json_access(lib, "name").GetString();

			// Check rules
			if (lib.HasMember("rules")) {
				bool rule_fail = false;
				for (auto& rule : lib["rules"].GetArray()) {
					const std::string_view action = json_access(rule, "action").GetString();
					const std::string_view os_name = json_access(json_access(rule, "os"), "name").GetString();
					if (
						(action == "allow" && os_name != OS_STRING) ||
						(action == "disallow" && os_name == OS_STRING)
					) {
						rule_fail = true;
					}
				}
				if (rule_fail) continue;
			}

			std::shared_ptr<DownloadHelper> lib_helper = std::make_shared<DownloadHelper>();
			lib_helper->set_options(DownloadHelper::FILE, url, path);
			multi_helper.add_helper(lib_helper);
			OUT_REPL(name);
		}
		OUT_NEWLINE();

		// Assets
		const fs::path assets_path = get_mcvm_dir() / "assets";
		create_dir_if_not_exists(assets_path / "indexes");
		const fs::path asset_index_path = assets_path / "indexes" / (version + ".json");

		std::string asset_index_contents = download_cached_file(json_access(json_access(ret, "assetIndex"), "url").GetString(), asset_index_path, true);

		create_dir_if_not_exists(assets_path / "objects");
		// TODO: Make a copy in virtual for old versions
		create_dir_if_not_exists(assets_path / "virtual");

		json::Document asset_index;
		asset_index.Parse<json::kParseStopWhenDoneFlag>(asset_index_contents.c_str());

		for (auto& asset_val : json_access(asset_index, "objects").GetObject()) {
			json::GenericObject asset = asset_val.value.GetObject();
			const std::string hash = json_access(asset, "hash").GetString();
			const std::string hash_path = hash.substr(0, 2) + '/' + hash;
			const fs::path path = assets_path / "objects" / hash_path;
			if (file_exists(path)) continue;
			const std::string url = std::string("http://resources.download.minecraft.net/" + hash_path);

			create_leading_directories(path);

			std::shared_ptr<DownloadHelper> asset_helper = std::make_shared<DownloadHelper>();
			asset_helper->set_options(DownloadHelper::FILE, url, path);
			multi_helper.add_helper(asset_helper);
		}

		// TODO: Natives

		OUT("Downloading libraries and assets...");
		multi_helper.perform_blocking();
	}
};
