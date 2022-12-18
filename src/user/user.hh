#pragma once

#include "skin.hh"

#include <string>
#include <ctype.h>
#include <stdexcept>

namespace mcvm {
	class User {
		public:
		std::string name = "";
	};

	struct InvalidUsernameException : public std::exception {
		const char* what() {
			return "Invalid username for an account";
		}
	};

	class MojangUser : public User {
		public:
		MojangUser(const std::string _username);

		std::string username = "";
		Skin skin;

		/*
			Returns true if a Mojang username is valid, and false otherwise
			Keep in mind that even though 3-character long usernames are no longer possible,
			they are still considered valid as usernames that long used to be possible and still exist 
		*/
		static const bool is_valid_username(const std::string username);
	};
};
