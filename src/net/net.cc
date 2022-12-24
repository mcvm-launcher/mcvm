#include "net.hh"

namespace mcvm {
	void net_start() {
		// Initialize curl with Win32 socket support
		curl_global_init(CURL_GLOBAL_WIN32);
	}

	void net_stop() {
		// Close down curl
		curl_global_cleanup();
	}

	std::size_t write_data_to_file(void* buffer, size_t size, size_t nmemb, void* file) {
		FILE* file_cast = static_cast<FILE*>(file);
		return fwrite(buffer, size, nmemb, file_cast);
	}

	std::size_t write_data_to_str(void* buffer, size_t size, size_t nmemb, void* str) {
		std::string* str_cast = static_cast<std::string*>(str);
		std::size_t write_size = size * nmemb;
		str_cast->append(static_cast<const char*>(buffer), write_size);
		return write_size;
	}
	
	std::size_t write_data_to_file_and_str(void* buffer, size_t size, size_t nmemb, void* curl_result) {
		CurlResult* result = static_cast<CurlResult*>(curl_result);
		size_t written = write_data_to_file(buffer, size, nmemb, result->file);
		// Append to the string
		write_data_to_str(buffer, size, nmemb, &result->str);

		return written;
	}
	
	CurlResult::~CurlResult() {
		fclose(file);
	}

	DownloadHelper::DownloadHelper(DownloadMode _mode, const std::string& url, const fs::path path)
	: mode(_mode) {
		handle = curl_easy_init();

		curl_easy_setopt(handle, CURLOPT_URL, url.c_str());

		curl_easy_setopt(handle, CURLOPT_ERRORBUFFER, errbuf);
		errbuf[0] = 0;

		curl_easy_setopt(handle, CURLOPT_WRITEFUNCTION, &write_data_to_file_and_str);
		curl_easy_setopt(handle, CURLOPT_WRITEDATA, &res);

		// TODO: Actually change function based on mode
		if (mode == DownloadMode::FILE || mode == DownloadMode::FILE_AND_STR) {
			res.file = fopen(path.c_str(), "wb");
			if (!res.file) {
				throw FileOpenError{};
			}
		}
	}

	bool DownloadHelper::perform() {
		CURLcode success = curl_easy_perform(handle);

		// We don't need to fclose since thats in the destructor for CurlResult, but we should put one here when switch based on mode
		curl_easy_cleanup(handle);

		if (success != CURLcode::CURLE_OK) {
			ERR(errbuf);
			return false;
		}
		return true;
	}

	std::string DownloadHelper::get_str() {
		return res.str;
	}

	std::string DownloadHelper::get_err() {
		return std::string(errbuf);
	}
};
