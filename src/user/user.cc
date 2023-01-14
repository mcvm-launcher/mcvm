#include "user.hh"

namespace mcvm {
	User::User(std::string _id) : id(_id) {}

	MicrosoftUser::MicrosoftUser(std::string _id, std::string _username)
	: User(_id), username(_username) {
		if (!is_valid_username(_username)) {
			throw InvalidUsernameException();
		}
	}

	bool MicrosoftUser::is_valid_username(std::string username) {
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
