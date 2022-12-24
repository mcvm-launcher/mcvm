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
	
	std::size_t write_data_to_file_and_str(void* buffer, size_t size, size_t nmemb, void* curl_result) {
		CurlResult* result = static_cast<CurlResult*>(curl_result);
		size_t written = write_data_to_file(buffer, size, nmemb, result->file);

		// Append to the str
		char* strbuf = static_cast<char*>(calloc(nmemb + 1, size));
		strcpy(strbuf, static_cast<const char*>(buffer));
		result->str += strbuf;

		return written;
	}
	
	CurlResult::~CurlResult() {
		fclose(file);
	}
};
