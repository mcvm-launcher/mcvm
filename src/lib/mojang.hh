#pragma once

#include <string>

namespace mcvm {
	// For checking rule actions in Mojang json files
	static inline bool is_allowed(const std::string& action) {
		return (action == "allow");
	}
};
