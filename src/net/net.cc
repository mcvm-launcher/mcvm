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
		size_t written = fwrite(buffer, size, nmemb, (FILE *)file);
		return written;
	}
};
