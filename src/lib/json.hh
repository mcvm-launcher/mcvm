#pragma once
#include "util.hh"

#include <rapidjson/document.h>
#include <rapidjson/filereadstream.h>
#include <rapidjson/filewritestream.h>
#include <rapidjson/prettywriter.h>

namespace mcvm {
	// Access a json value with an assertion that it is there
	static inline json::Value& json_access(json::Value& val, const char* key) {
		assert(val.HasMember(key));
		return val[key];
	}

	static inline json::Value& json_access(json::Value* val, const char* key) {
		assert(val->HasMember(key));
		return val->operator[](key);
	}

	static inline json::Value& json_access(json::GenericObject<false, json::Value>& val, const char* key) {
		assert(val.HasMember(key));
		return val[key];
	}

	static inline void json_read(json::Document& doc, const fs::path& path) {
		FILE* file = fopen(path.c_str(), "rb");
		char readbuf[CHARBUF_LARGE];
		json::FileReadStream st(file, readbuf, sizeof(readbuf));
		json::AutoUTFInputStream<unsigned, json::FileReadStream> ist(st);
		doc.ParseStream<0, json::AutoUTF<unsigned>>(ist);
	}

	static inline void json_write(json::Document& doc, const fs::path& path, bool format = true) {
		FILE* file = fopen(path.c_str(), "wb");
		char writebuf[CHARBUF_LARGE];
		json::FileWriteStream os(file, writebuf, sizeof(writebuf));
		
		json::PrettyWriter writer(os);
		if (format) {
			writer.SetIndent('\t', 1);
		}
		doc.Accept(writer);
		fclose(file);
	}
};
