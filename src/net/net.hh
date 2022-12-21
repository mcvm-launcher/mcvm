#pragma once
#include "io/files.hh"

#include <curl/curl.h>
#include <rapidjson/rapidjson.h>

#include <iostream>
#include <fstream>
#include <assert.h>

namespace mcvm {
	// Start / initialize networking stuff
	extern void net_start();
	// Stop networking stuff
	extern void net_stop();

	// Updates asset and library indexes with Mojang servers
	extern void update_assets();

	extern std::size_t write_data_to_file(void* buffer, size_t size, size_t nmemb, void* file);
};
