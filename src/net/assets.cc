#include "net.hh"

namespace mcvm {
	// README: https://wiki.vg/Game_files

	std::shared_ptr<DownloadHelper> get_version_manifest(const CachedPaths& paths, bool verbose) {
		create_dir_if_not_exists(paths.assets);
		create_dir_if_not_exists(paths.internal / "versions");

		if (verbose) OUT("\tObtaining version index...");

		const fs::path manifest_file_path = paths.internal / "versions" / "version_manifest.json";
		std::shared_ptr<DownloadHelper> helper = std::make_shared<DownloadHelper>();
		helper->set_options(DownloadHelper::FILE_AND_STR, VERSION_MANIFEST_URL, manifest_file_path);
		helper->perform();
		return helper;
	}

	std::shared_ptr<DownloadHelper> obtain_version_json(
		const MCVersionString& version,
		json::Document* ret,
		const CachedPaths& paths,
		bool verbose
	) {
		if (verbose) OUT_LIT("\tDownloading version json...");
		std::shared_ptr<DownloadHelper> helper = get_version_manifest(paths, verbose);
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
		create_dir_if_not_exists(paths.internal / "versions" / version);
		const fs::path index_file_path = paths.internal / "versions" / version / fs::path(index_file_name);
		helper->set_options(DownloadHelper::FILE_AND_STR, ver_url, index_file_path);
		helper->set_checksum(ver_hash);
		helper->perform();
		helper->perform_checksum();
		ret->Parse(helper->get_str().c_str());

		return helper;
	}

	void download_assets(
		json::Document* ret,
		const CachedPaths& paths,
		std::shared_ptr<DownloadHelper> helper,
		MultiDownloadHelper& multi_helper,
		const std::string& version_string,
		bool verbose
	) {
		const fs::path indexes_path = paths.assets / ASSETS_INDEXES_DIR;
		create_dir_if_not_exists(indexes_path);
		const fs::path asset_index_path = indexes_path / (version_string + ".json");

		const std::string asset_index_url = json_access(json_access(ret, "assetIndex"), "url").GetString();
		std::string asset_index_contents = download_cached_file(asset_index_url, asset_index_path, true, helper);

		json::Document asset_index;
		asset_index.Parse<json::kParseStopWhenDoneFlag>(asset_index_contents.c_str());
		json::ParseErrorCode error_code = asset_index.GetParseError();
		if (error_code != json::kParseErrorNone) {
			WARN("Asset index was malformed, redownloading...");
			asset_index_contents = download_cached_file(asset_index_url, asset_index_path, true, helper);
			asset_index.Parse<json::kParseStopWhenDoneFlag>(asset_index_contents.c_str());
		}

		const fs::path assets_objects_path = paths.assets / ASSETS_OBJECTS_DIR;
		const fs::path assets_virtual_path = paths.assets / ASSETS_VIRTUAL_DIR;
		create_dir_if_not_exists(assets_objects_path);
		if (!fs::exists(assets_virtual_path)) {
			fs::create_directory_symlink(assets_objects_path, assets_virtual_path);
		}

		json::GenericObject assets = json_access(asset_index, "objects").GetObject();

		if (verbose) {
			OUT("\tFound " << BLUE(assets.MemberCount()) << " assets...");
		}

		// We have to batch these to prevent going over the file descriptor limit
		static const uint batch_size = 128;
		uint batch_index = 0;
		uint batch_count = 0;

		for (auto& asset_val : assets) {
			json::GenericObject asset = asset_val.value.GetObject();
			const std::string hash = json_access(asset, "hash").GetString();
			const std::string hash_path = hash.substr(0, 2) + '/' + hash;
			const fs::path path = assets_objects_path / hash_path;
			if (file_exists(path)) continue;
			const std::string url = std::string("http://resources.download.minecraft.net/" + hash_path);

			create_leading_directories(path);

			if (batch_index > batch_size) {
				if (verbose) OUT_REPL(
					GRAY_START << "\t\tDownloading batch "
					<< BLUE_START << batch_count << GRAY("...")
				);
				multi_helper.perform_blocking();
				batch_index = 0;
				batch_count++;
			}

			std::shared_ptr<DownloadHelper> asset_helper = std::make_shared<DownloadHelper>();
			asset_helper->set_options(DownloadHelper::FILE, url, path);
			multi_helper.add_helper(asset_helper);
			batch_index++;
		}

		if (verbose) OUT_NEWLINE();
	}

	// TODO: Make this a string instead so we don't store so many redundant paths
	void install_native_library(const fs::path& path, const fs::path& natives_dir) {
		int zip_err = ZIP_ER_OK;
		zip* jar_file = zip_open(path.c_str(), ZIP_RDONLY, &zip_err);
		if (zip_err != ZIP_ER_OK) {
			ERR("Failed to open jar file with error code " << zip_err);
			if (zip_err == ZIP_ER_NOZIP) {
				ERR("File is not a zip archive");
			}
			return;
		}

		zip_stat_t file_stat;
		zip_file* file;
		for (uint i = 0; i < zip_get_num_entries(jar_file, 0); i++) {
			if (zip_stat_index(jar_file, i, 0, &file_stat) == 0) {
				fs::path name_path = file_stat.name;
				name_path = name_path.filename();
				if (
					name_path.extension() != ".so"
					&& name_path.extension() != ".dylib"
					&& name_path.extension() != ".dll"
				) continue;

				file = zip_fopen_index(jar_file, i, 0);
				char* contents = new char[file_stat.size];
				zip_fread(file, contents, file_stat.size);
				write_file(natives_dir / name_path, contents);
			}
		}

		zip_close(jar_file);
	}

	std::shared_ptr<DownloadHelper> obtain_libraries(
		MinecraftVersion version,
		json::Document* ret,
		const CachedPaths& paths,
		std::string& classpath,
		bool verbose
	) {
		const MCVersionString version_string = mc_version_reverse_map.at(version);
		
		std::shared_ptr<DownloadHelper> helper = obtain_version_json(version_string, ret, paths, verbose);

		const fs::path libraries_path = paths.internal / "libraries";
		create_dir_if_not_exists(libraries_path);
		const fs::path natives_path = paths.internal / "versions" / version_string / "natives";
		create_dir_if_not_exists(natives_path);
		const fs::path native_jars_path = paths.internal / "natives";

		if (verbose) OUT_LIT("\tFinding libraries...");

		MultiDownloadHelper multi_helper;

		std::vector<fs::path> native_libs;

		for (auto& lib_val : json_access(ret, "libraries").GetArray()) {
			json::GenericObject lib = lib_val.GetObject();
			
			// Check rules
			if (lib.HasMember("rules")) {
				bool rule_fail = false;
				for (auto& rule : lib["rules"].GetArray()) {
					const std::string action = json_access(rule, "action").GetString();
					if (rule.HasMember("os")) {
						const std::string_view os_name = json_access(rule["os"], "name").GetString();
						if (
							(is_allowed(action) != (os_name == OS_STRING))
						) {
							rule_fail = true;
						}
					}
				}
				if (rule_fail) continue;
			}

			const std::string name = json_access(lib, "name").GetString();
			if (lib.HasMember("natives") && lib["natives"].HasMember(OS_STRING)) {
				const std::string natives_key = lib["natives"][OS_STRING].GetString();
				json::GenericObject classifiers = json_access(
					json_access(lib, "downloads"), "classifiers"
				).GetObject();
				json::GenericObject classifier = json_access(classifiers, natives_key.c_str()).GetObject();
				const std::string path_str = json_access(classifier, "path").GetString();

				fs::path path = native_jars_path / path_str;
				create_leading_directories(path);
				native_libs.push_back(path);
				classpath += path.c_str();
				classpath += ':';

				const std::string url = json_access(classifier, "url").GetString();
				const std::string hash = json_access(classifier, "sha1").GetString();
				std::shared_ptr<DownloadHelper> native_helper = std::make_shared<DownloadHelper>();
				native_helper->set_options(DownloadHelper::FILE, url, path);
				native_helper->set_checksum(hash);
				multi_helper.add_helper(native_helper);
			}

			if (!lib.HasMember("downloads")) return helper;
			if (!lib["downloads"].HasMember("artifact")) return helper;
			json::GenericObject download_artifact = json_access(json_access(lib, "downloads"), "artifact").GetObject();
			const char* path_str =  json_access(download_artifact, "path").GetString();
			fs::path path = libraries_path / path_str;

			classpath += path.c_str();
			classpath += ':';

			// If we already have the library don't download it again
			if (file_exists(path)) continue;
			create_leading_directories(path);

			const char* url = json_access(download_artifact, "url").GetString();
			const std::string hash = json_access(download_artifact, "sha1").GetString();

			std::shared_ptr<DownloadHelper> lib_helper = std::make_shared<DownloadHelper>();
			lib_helper->set_options(DownloadHelper::FILE, url, path);
			lib_helper->set_checksum(hash);
			multi_helper.add_helper(lib_helper);
			if (verbose) OUT("\t\tFound library " << name);
		}

		if (verbose) OUT("\tDownloading " << BLUE(multi_helper.get_helper_count()) << " libraries...");
		multi_helper.perform_blocking();

		download_assets(ret, paths, helper, multi_helper, version_string, verbose);

		// Deal with proper installation of native libraries now that we have them
		if (verbose) OUT_LIT("\tExtracting natives...");
		for (uint i = 0; i < native_libs.size(); i++) {
			install_native_library(native_libs[i], natives_path);
		}

		return helper;
	}
};
