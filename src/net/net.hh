#pragma once
#include "io/files.hh"
#include "data/info.hh"
#include "lib/util.hh"

#include <curl/curl.h>
#include <rapidjson/document.h>

#include <iostream>
#include <assert.h>
#include <memory>
#include <vector>

// URLs
#define VERSION_MANIFEST_URL "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
#define MOJANG_LIBRARIES_URL "https://libraries.minecraft.net/"

namespace json = rapidjson;

namespace mcvm {
	// A struct passed in file writing from curl that holds both a file ptr and a string buffer to write into
	struct CurlResult {
		FILE* file = nullptr;
		std::string str = "";

		~CurlResult();
	};

	// Start / initialize networking stuff
	extern void net_start();
	// Stop networking stuff
	extern void net_stop();

	// Updates asset and library indexes with Mojang servers
	// Returns the manifest json file contents
	extern std::string update_assets();

	// Obtain the json file for a version
	extern void obtain_version_json(const std::string& version, json::Document* ret);

	// Obtain libraries for a version
	extern void obtain_libraries(const std::string& version, json::Document* ret);

	// Callback response for curl perform that writes the data to a file
	extern std::size_t write_data_to_file(void* buffer, size_t size, size_t nmemb, void* file);

	// Callback response for curl perform that writes the data to a string
	extern std::size_t write_data_to_str(void* buffer, size_t size, size_t nmemb, void* str);

	// Callback response for curl perform that writes the data to a file and a string
	extern std::size_t write_data_to_file_and_str(void* buffer, size_t size, size_t nmemb, void* curl_result);

	class MultiDownloadHelper;

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

		void set_options(DownloadMode mode, const std::string& url, const fs::path& path);
		bool perform();

		std::string get_str();
		std::string get_err();

		~DownloadHelper();

		private:
		CURL* handle;
		char errbuf[CURL_ERROR_SIZE];
		CurlResult res;

		friend class MultiDownloadHelper;
	};

	// Wrapper around a libcurl multi handle
	class MultiDownloadHelper {
		CURLM* handle;
		std::vector<std::shared_ptr<DownloadHelper>> helpers;
		int is_performing = 1;
		int messages_left;
		
		public:
		MultiDownloadHelper();

		// Add a download helper to the multi
		void add_helper(std::shared_ptr<DownloadHelper> helper);
		// Do the performs (blocking)
		bool perform_blocking();

		~MultiDownloadHelper();
	};
};
