#include <iostream>
#include "user/user.hh"

int main(int argc, char** argv) {
	try {
		mcvm::MojangUser user("ItsaMe123");
	} catch (mcvm::InvalidUsernameException& e) {
		std::cerr << e.what() << "\n";
	}

	return 0;
}
