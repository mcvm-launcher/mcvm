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

	DownloadHelper::DownloadHelper() {
		handle = curl_easy_init();

		curl_easy_setopt(handle, CURLOPT_ERRORBUFFER, errbuf);
		errbuf[0] = 0;
	}

	void DownloadHelper::set_options(DownloadMode mode, const std::string& url, const fs::path& path) {
		curl_easy_setopt(handle, CURLOPT_URL, url.c_str());

		// Reset the result
		res.str = "";
		if (res.file) {
			fclose(res.file);
		}

		if (mode == DownloadMode::FILE || mode == DownloadMode::FILE_AND_STR) {
			res.file = fopen(path.c_str(), "wb");
			if (!res.file) {
				throw FileOpenError{path.c_str()};
			}
		}

		switch (mode) {
			case DownloadMode::FILE:
				curl_easy_setopt(handle, CURLOPT_WRITEFUNCTION, &write_data_to_file);
				curl_easy_setopt(handle, CURLOPT_WRITEDATA, res.file);
				break;
			case DownloadMode::STR:
				curl_easy_setopt(handle, CURLOPT_WRITEFUNCTION, &write_data_to_str);
				curl_easy_setopt(handle, CURLOPT_WRITEDATA, &res.str);
				break;
			case DownloadMode::FILE_AND_STR:
				curl_easy_setopt(handle, CURLOPT_WRITEFUNCTION, &write_data_to_file_and_str);
				curl_easy_setopt(handle, CURLOPT_WRITEDATA, &res);
				break;
		}
	}

	bool DownloadHelper::perform() {
		CURLcode success = curl_easy_perform(handle);

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

	DownloadHelper::~DownloadHelper() {
		// We don't need to fclose since thats in the destructor for CurlResult, but we should put one here when switch based on mode
		curl_easy_cleanup(handle);
	}

	MultiDownloadHelper::MultiDownloadHelper() {
		handle = curl_multi_init();
	}

	void MultiDownloadHelper::add_helper(std::shared_ptr<DownloadHelper> helper) {
		helpers.push_back(helper);
		curl_multi_add_handle(handle, helper->handle);
	}

	bool MultiDownloadHelper::perform_blocking() {
		while (is_performing) {
			CURLMcode code = curl_multi_perform(handle, &is_performing);

			if (is_performing) {
				code = curl_multi_poll(handle, NULL, 0, 1000, NULL);
			}

			if (code) {
				break;
			}
		}
		// TODO: Error handling and messages for multi helper
		return true;
	}

	MultiDownloadHelper::~MultiDownloadHelper() {
		for (unsigned int i = 0; i < helpers.size(); i++) {
			curl_multi_remove_handle(handle, helpers[i]->handle);
		}
		curl_multi_cleanup(handle);
	}
};
