#include "files.hh"

namespace mcvm {
	bool file_exists(const fs::path& path) {
		struct stat buffer;
		return (stat (path.c_str(), &buffer) == 0); 
	}

	void create_dir_if_not_exists(const fs::path& path) {
		fs::create_directory(path);
		// if (!fs::is_directory(path) && !fs::exists(path)) {
		// }
	}

	std::size_t path_length(const fs::path& path) {
		std::size_t len = 0;
		for (auto i = path.begin(); i != path.end(); i++) {
			len += 1;
		}
		return len;
	}

	void create_leading_directories(const fs::path& path) {
		// Remove the final entry in the path
		const fs::path split_path = path.parent_path();
		// We need to continuously append to this path so that the full paths are there instead of relative ones
		fs::path full_path;
		for (auto i = split_path.begin(); i != split_path.end(); i++) {
			full_path /= *i;
			create_dir_if_not_exists(full_path);
		}
	}

	void read_file(const fs::path& path, std::string& out) {
		std::ifstream file(path);
		std::string line;
		if (file.is_open()) {
			while (file.good()) {
				std::getline(file, line);
				out += line;
			}
		} else {
			throw FileOpenError{path};
		}
	}

	void write_file(const fs::path& path, const char* text) {
		FILE* file = fopen(path.c_str(), "w");
		if (file) {
			fwrite(text, sizeof(char), strlen(text), file);
			fclose(file);
		} else {
			throw FileOpenError{path};
		}
	}

	void extract_tar_gz(const fs::path& path) {
		gzFile gz_file = gzopen(path.c_str(), "r");
		fs::path tar_path = path;
		tar_path.replace_extension(".tar");
		FILE* tar_file = fopen(tar_path.c_str(), "wb");
		if (!tar_file) {
			gzclose_r(gz_file);
			throw FileOpenError{tar_path, errno};
		}
		// This will be a large file so we have to do this incrementally
		static const uint buf_size = CHARBUF_LARGE;
		char buf[CHARBUF_LARGE];
		int write_n;
		while(true) {
			write_n = gzread(gz_file, &buf, buf_size);
			if (write_n == 0) break;
			if (write_n < 0) {
				ERR("Failed to decompress " << path);
				ERR("Error code: " << gzerror(gz_file, NULL));
				exit(1);
			}
			fwrite(&buf, sizeof(char), write_n, tar_file);
		}
		gzclose_r(gz_file);
		fclose(tar_file);

		char extract_to[CHARBUF_SMALL];
		strcpy(extract_to, path.parent_path().c_str());

		TAR* tar_extract_file;
		tar_open(&tar_extract_file, tar_path.c_str(), NULL, 0, 0, 0);
		tar_extract_all(tar_extract_file, extract_to);

		tar_close(tar_extract_file);
	}
};
