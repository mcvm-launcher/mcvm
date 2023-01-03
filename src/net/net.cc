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

	std::size_t write_data_to_file(void* buffer, size_t size, size_t nmemb, void* curl_result) {
		FILE* file = static_cast<CurlResult*>(curl_result)->file;
		return fwrite(buffer, size, nmemb, file);
	}

	std::size_t write_data_to_str(void* buffer, size_t size, size_t nmemb, void* curl_result) {
		std::string* str = &static_cast<CurlResult*>(curl_result)->str;
		std::size_t write_size = size * nmemb;
		str->append(static_cast<const char*>(buffer), write_size);
		return write_size;
	}
	
	std::size_t write_data_to_file_and_str(void* buffer, size_t size, size_t nmemb, void* curl_result) {
		CurlResult* result = static_cast<CurlResult*>(curl_result);
		size_t written = write_data_to_file(buffer, size, nmemb, result);
		write_data_to_str(buffer, size, nmemb, result);

		return written;
	}

	// TODO: Unfinished
	int progress_callback(void* clientp, double dltotal, double dlnow, double ultotal, double ulnow) {
		ProgressData* data = static_cast<ProgressData*>(clientp);
		static unsigned int intervals = 10;
		if (dltotal != 0 && dlnow != 0) {
			std::string bar = "";
			const unsigned int count = round(dlnow / dltotal * intervals);
			for (unsigned int i = 0; i < intervals; i++) {
				if (i < count) {
					bar.push_back('.');
				} else {
					bar.push_back(' ');
				}
			}
			OUT_REPL(data->title << '[' << bar << ']');
		}

		return CURL_PROGRESSFUNC_CONTINUE;
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
		// Reset
		curl_easy_reset(handle);
		res.str = "";
		if (res.file) {
			fclose(res.file);
		}

		curl_easy_setopt(handle, CURLOPT_URL, url.c_str());

		if (mode == DownloadMode::FILE || mode == DownloadMode::FILE_AND_STR) {
			res.file = fopen(path.c_str(), "wb");
			if (!res.file) {
				throw FileOpenError{path.c_str()};
			}
		}

		switch (mode) {
			case DownloadMode::FILE:
				curl_easy_setopt(handle, CURLOPT_WRITEFUNCTION, &write_data_to_file);
				break;
			case DownloadMode::STR:
				curl_easy_setopt(handle, CURLOPT_WRITEFUNCTION, &write_data_to_str);
				break;
			case DownloadMode::FILE_AND_STR:
				curl_easy_setopt(handle, CURLOPT_WRITEFUNCTION, &write_data_to_file_and_str);
				break;
		}
		curl_easy_setopt(handle, CURLOPT_WRITEDATA, &res);
	}

	bool DownloadHelper::perform() {
		CURLcode success = curl_easy_perform(handle);

		if (progress_data.is_used) {
			OUT_NEWLINE();
		}

		if (success != CURLcode::CURLE_OK) {
			ERR(errbuf);
			return false;
		}
		return true;
	}

	bool DownloadHelper::sha1_checksum(const std::string& checksum) {
		// TODO: Temporary
		return true;
	}

	void DownloadHelper::add_progress_meter(ProgressData::ProgressStyle style, const std::string &title) {
		// progress_data.is_used = true;
		// progress_data.style = style;
		// progress_data.title = title;
		// curl_easy_setopt(handle, CURLOPT_NOPROGRESS, 0);
		// curl_easy_setopt(handle, CURLOPT_XFERINFOFUNCTION, &progress_callback);
		// curl_easy_setopt(handle, CURLOPT_PROGRESSDATA, progress_data);
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

			if (code) break;
		}
		// TODO: Error handling and messages for multi helper
		return true;
	}

	void MultiDownloadHelper::reset() {
		for (unsigned int i = 0; i < helpers.size(); i++) {
			curl_multi_remove_handle(handle, helpers[i]->handle);
		}
		helpers = {};
	}

	void MultiDownloadHelper::add_progress_meter(ProgressData::ProgressStyle style, const std::string &title) {
		// progress_data.style = style;
		// progress_data.title = title;
		// curl_multi_setopt(handle, CURLOPT_PROGRESSFUNCTION, &progress_callback);
		// curl_multi_setopt(handle, CURLOPT_PROGRESSDATA, progress_data);
	}

	MultiDownloadHelper::~MultiDownloadHelper() {
		reset();
		curl_multi_cleanup(handle);
	}

	std::string download_cached_file(const std::string& url, const fs::path& path, bool download_str) {
		if (file_exists(path)) {
			if (download_str) {
				std::string ret;
				read_file(path, ret);
				return ret;
			} else {
				return "";
			}
		} else {
			DownloadHelper helper;
			DownloadHelper::DownloadMode mode;
			if (download_str) {
				mode = DownloadHelper::FILE_AND_STR;
			} else {
				mode = DownloadHelper::FILE;
			}
			helper.set_options(mode, url, path);
			helper.perform();
			return helper.get_str();
		}
	}
};
