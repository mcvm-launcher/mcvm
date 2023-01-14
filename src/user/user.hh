#pragma once
#include "lib/util.hh"
#include "skin.hh"
#include "net/net.hh"

#include <string>
#include <ctype.h>
#include <stdexcept>

namespace mcvm {
	class User {
		public:
		const std::string id;

		User(std::string _id);

		virtual bool is_demo() {
			return false;
		}
	};

	struct InvalidUsernameException : public std::exception {
		const char* what() {
			return "Invalid username for an account";
		}
	};

	class MicrosoftUser : public User {
		public:
		MicrosoftUser(std::string _id, std::string _username, std::string _uuid = "");

		std::string username;
		std::string uuid;
		Skin skin;

		/*
			Returns true if a Mojang username is valid, and false otherwise
			Keep in mind that even though 3-character long usernames are no longer possible,
			they are still considered valid as usernames that long used to be possible and still exist 
		*/
		static bool is_valid_username(std::string username);

		// Grabs the UUID from the Mojang API
		void ensure_uuid();
	};

	class DemoUser : public User {
		public:
		using User::User;

		bool is_demo() override {
			return true;
		}
	};
};
