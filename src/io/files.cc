#include "files.hh"

namespace mcvm {
	bool file_exists(const std::string& path) {
		struct stat buffer;
		return (stat (path.c_str(), &buffer) == 0); 
	}

	void create_dir_if_not_exists(const fs::path& path) {
		if (!fs::is_directory(path) && !fs::exists(path)) {
			fs::create_directory(path);
		}
	}

	std::size_t path_length(const fs::path& path) {
		std::size_t len = 0;
		for (fs::path::const_iterator i = path.begin(); i != path.end(); i++) {
			len += 1;
		}
		return len;
	}

	void create_leading_directories(const fs::path& path) {
		// Remove the final entry in the path
		const fs::path split_path = path.parent_path();
		// We need to continuously append to this path so that the full paths are there instead of relative ones
		fs::path full_path;
		for (fs::path::const_iterator i = split_path.begin(); i != split_path.end(); i++) {
			full_path /= *i;
			create_dir_if_not_exists(full_path);
		}
	}

	void read_file(const fs::path& path, std::string& out) {
		std::ifstream file(path);
		if (file.is_open()) {
			while (file.good()) {
				const char appendbuf = file.get();
				out += appendbuf;
			}
		} else {
			throw FileOpenError{};
		}
	}

	void write_file(const fs::path& path, const char* text) {
		FILE* file = fopen(path.c_str(), "w");
		if (file) {
			fwrite(text, sizeof(char), strlen(text), file);
			fclose(file);
		} else {
			throw FileOpenError{};
		}
	}
};
