import urllib.request
import json
import re

urllib.request.urlretrieve("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json", "vmanifest.json")

with open("vmanifest.json", "r") as file:
	man = json.loads(file.read())
	versions = man["versions"]
	version_names = []
	version_names_raw = []
	for version in versions:
		id = str(version["id"])
		version_names_raw.append(id)

		id = id.upper()
		id = re.sub("(\.|-| )", "_", id)
		# We need this since enum names can't start with numbers
		id = "V_" + id

		assert(not id in version_names)
		version_names.append(id)

	# The versions are reversed so that their order makes it simple to compare them
	# e.g. 
	version_names.reverse()
	version_names_raw.reverse()

	names_join = ",\n".join(version_names)
	names_enum = "\tenum class MinecraftVersion {\n\t\t" + ",\n\t\t".join(version_names) + "\n\t};"
	
	forward_map_values = []
	for name, raw in zip(version_names, version_names_raw):
		forward_map_values.append('{"' + raw + '", MinecraftVersion::' + name + "}")
	forward_map = "\tstatic std::map<std::string, MinecraftVersion> mc_version_forward_map = {\n\t\t" + ",\n\t\t".join(forward_map_values) + "\n\t};"

	reverse_map_values = []
	for name, raw in zip(version_names, version_names_raw):
		reverse_map_values.append('{' + "MinecraftVersion::" + name + ', "' + raw + '"}')
	reverse_map = "\tstatic std::map<MinecraftVersion, std::string> mc_version_reverse_map = {\n\t\t" + ",\n\t\t".join(reverse_map_values) + "\n\t};"

	with open("versions.hh", "w") as outfile:
		outfile.write("#pragma once\n")
		outfile.write("#include <map>\n")
		outfile.write("#include <string>\n")
		outfile.write("\n")

		outfile.write("namespace mcvm {\n")
		outfile.write(names_enum)
		outfile.write("\n\n")
		outfile.write(forward_map)
		outfile.write("\n\n")
		outfile.write(reverse_map)
		outfile.write("\n};\n")
