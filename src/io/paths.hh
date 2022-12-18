#pragma once
#include <string>

#define PATH_SEP "/"
#define PATH_CONCAT_2(path1, path2) path1 PATH_SEP path2
#define PATH_CONCAT_3(path1, path2, path3) PATH_CONCAT_2(PATH_CONCAT_2(path1, path2), path3)
#define PATH_CONCAT_4(path1, path2, path3, path4) PATH_CONCAT_2(PATH_CONCAT_3(path1, path2, path3), path4)
#define PATH_CONCAT_5(path1, path2, path3, path4, path5) PATH_CONCAT_2(PATH_CONCAT_4(path1, path2, path3, path4), path5)
#define PATH_CONCAT_6(path1, path2, path3, path4, path5, path6) PATH_CONCAT_2(PATH_CONCAT_5(path1, path2, path3, path4, path5), path6)

#define HOME_DIR "~"
#define MCVM_DIR PATH_CONCAT_4(HOME_DIR, ".local", "share", "mcvm")
#ifdef WIN32
	// TODO: Actual path with user detection, in appdata or something
	#define MCVM_DIR PATH_CONCAT_2("C:", "mcvm")
#endif

namespace mcvm {
	static const std::string path_concat(const std::string& str1, const std::string& str2) {
		return str1 + PATH_SEP + str2;
	}
};
