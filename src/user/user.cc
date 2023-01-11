#include "user.hh"

namespace mcvm {
	MojangUser::MojangUser(const std::string _username) : username(_username) {
		if (!is_valid_username(_username)) {
			throw InvalidUsernameException();
		}
	}

	bool MojangUser::is_valid_username(const std::string username) {
		const std::size_t len = username.size();
		if (len > 16) {
			return false;
		}

		for (std::size_t i = 0; i < len; i++) {
			if (
				!isalnum(username[i]) &&
				username[i] != '_'
			) {
				return false;
			}
		}
		return true;
	}
};
