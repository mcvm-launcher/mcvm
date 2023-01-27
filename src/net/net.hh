#pragma once
#include "io/files/files.hh"
#include "data/info.hh"
#include "lib/json.hh"
#include "lib/mojang.hh"
#include "lib/versions.hh"
#include "sha1.hh"

#include <curl/curl.h>
#include <rapidjson/document.h>
#include <zip.h>

#include <iostream>
#include <assert.h>
#include <memory>
#include <vector>
#include <cmath>

// URLs
#define VERSION_MANIFEST_URL "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
#define MOJANG_LIBRARIES_URL "https://libraries.minecraft.net/"

namespace json = rapidjson;

namespace mcvm {
	class MultiDownloadHelper;
	class DownloadHelper;

	// Start / initialize networking stuff
	extern void net_start();
	// Stop networking stuff
	extern void net_stop();

	// Updates asset and library indexes with Mojang servers
	extern std::shared_ptr<DownloadHelper> get_version_manifest(
		const CachedPaths& paths, bool verbose = true
	);

	// Obtain the json file for a version
	extern std::shared_ptr<DownloadHelper> obtain_version_json(
		const MCVersionString& version,
		json::Document* ret,
		const CachedPaths& paths,
		bool verbose = true
	);

	// Obtain libraries for a version
	extern std::shared_ptr<DownloadHelper> obtain_libraries(
		MinecraftVersion version,
		json::Document* ret,
		const CachedPaths& paths,
		std::string& classpath,
		bool verbose = true,
		bool force = false
	);

	void obtain_assets(
		json::Document* version_json,
		const MinecraftVersion& version,
		std::shared_ptr<DownloadHelper> helper,
		const CachedPaths& paths,
		bool verbose,
		bool force = false
	);

	// CURL callbacks

	// A struct passed in file writing from curl that holds both a file ptr and a string buffer to write into
	struct CurlResult {
		FILE* file = nullptr;
		std::string str = "";
	};

	// Callback response for curl perform that writes the data to a file
	extern std::size_t write_data_to_file(void* buffer, size_t size, size_t nmemb, void* file);

	// Callback response for curl perform that writes the data to a string
	extern std::size_t write_data_to_str(void* buffer, size_t size, size_t nmemb, void* str);

	// Callback response for curl perform that writes the data to a file and a string
	extern std::size_t write_data_to_file_and_str(void* buffer, size_t size, size_t nmemb, void* curl_result);

	// Struct passed to download progress callback
	struct ProgressData {
		enum ProgressStyle {
			DEFAULT
		};

		ProgressStyle style;
		std::string title;
		// Used by helpers
		bool is_used = false;
	};

	// Callback for progress
	int progress_callback(void* clientp, double dltotal, double dlnow, double ultotal, double ulnow);

	struct FileValidateException : public std::exception {
		std::string what() {
			return NICE_STR_CAT("File did not pass checksum");
		}
	};

	// Wrapper around a libcurl handle
	class DownloadHelper {
		public:
		// Option for what data should be obtained when downloading
		enum DownloadMode {
			FILE,
			STR,
			FILE_AND_STR
		};

		DownloadHelper();

		void set_options(DownloadMode _mode, const std::string& url, const fs::path& _path = "/");
		void follow_redirect();
		void set_verbose(const CachedPaths& paths);
		bool perform();
		void reset();
		void set_checksum(const std::string& _checksum);
		void perform_checksum(SHA1* sha1 = nullptr);
		void add_progress_meter(ProgressData::ProgressStyle style, const std::string& title);

		const std::string& get_str();
		std::string get_err();
		long get_response_code();
		void log_results();

		~DownloadHelper();

		private:
		CURL* handle;
		char errbuf[CURL_ERROR_SIZE];
		DownloadMode mode;
		fs::path path;
		CurlResult res;
		ProgressData progress_data;
		std::string checksum = "";

		friend class MultiDownloadHelper;
	};

	// Wrapper around a libcurl multi handle
	class MultiDownloadHelper {
		CURLM* handle;
		// This is pretty stupid but this lets us clean up a helper when it is done
		std::map<CURL*, std::shared_ptr<DownloadHelper>> helpers;
		int msgs_in_queue = 1;
		ProgressData progress_data;
		
		public:
		MultiDownloadHelper();

		// Add a download helper to the multi
		void add_helper(std::shared_ptr<DownloadHelper> helper);
		// Do the performs (blocking)
		bool perform_blocking();
		// Reset to prepare another multi transfer
		void reset();
		// Add a progress meter
		void add_progress_meter(ProgressData::ProgressStyle style, const std::string& title);
		// Set a limit to the number of concurrent connections. Useful for avoid file descriptor limits
		void set_connection_limit(ulong limit);
		// How many helpers are in the multi helper
		std::size_t get_helper_count();

		~MultiDownloadHelper();
	};

	// Download a file if it is not already cached locally
	extern std::string download_cached_file(const std::string& url, const fs::path& path, bool download_str = false, std::shared_ptr<DownloadHelper> helper = std::make_shared<DownloadHelper>());
};
