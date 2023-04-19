# Packages
An MCVM package is simply a script that is run to install files and dependencies. The file usually follows the format of `package-name.pkg.txt`. Package names may contain only letters, numbers, and hyphens (`-`).

# Repository
A package repository is any server that provides an `index.json` of packages for the user to source. All that is required to run a repository yourself is to make this `index.json` under `https://example.com/api/mcvm/index.json`. An index follows this format:
```json
{
	"packages": {
		"package-name": {
			"version": Integer,
			"url": String
		}
	}
}
```
 * `package-name`: The name of the package.
 * `version`: The package version known by the repository.
 * `url`: The URL to the `package.pkg.txt` file.

# Syntax
At the root level, a package is organized into **routines** which describe a list of instructions to be run to perform some action. Routines can have any name, but some have special meaning.
```
@routine_name {
	...
}
```

The main routine that will be in every single package is the `@install` routine. This routine is run when the package is installed or updated in order to download files for your game.

## Instructions
Instructions are individual commands that are run inside routines for your package script. Instructions are separated by semicolons. They often have arguments that can either be an identifier or a string. For any argument that takes a `"string"`, you can instead put `$variable` to substitute whatever the value of that variable is.

 * `if {condition} [arguments...] { ... }`: If instructions let you run instructions inside a block only if a condition is met at runtime. The valid conditions are:
	 * `value {x} {y}`: Check if two strings are the same. This is meant to be used to check the value of variables.
	 * `version {pattern}`: Check that the Minecraft version of this instance matches a pattern.
	 * `modloader {vanilla | fabric | forge | quilt | fabriclike}`: Checks if the modloader supports a mod type. The `fabriclike` option will match both Fabric and Quilt and should be used for most Fabric mods unless you know they don't play nice with Quilt.
	 * `plugin_loader {vanilla | bukkit}`: Checks if the plugin loader supports a plugin type.
	 * `side {client | server}`: Check what instance type the package is being installed on.
	 * `feature {name}`: Check if a feature is enabled for this package.
	 * `not {condition}`: Inverts a condition. You can chain these, but why would you want to.
 * `set {variable} {value}`: Sets the value of a variable.
 * `finish`: Usually put in side checks, will silently end the evaluation of the routine.
 * `fail [unsupported_version | unsupported_modloader | unsupported_plugin_loader]`: End execution with an error.
 * `addon {id} {filename} (..)`: Add an addon to the instance. This is the main goal of a package. The name field is the filename of the addon. Keys and values are put inside the parentheses.

## The addon Instruction
The `addon` instruction is a bit more complex. Inside the parentheses you put a set of keys and values to configure the addon and how it is installed. The full addon config looks like this:
```
addon id filename (
	kind: mod | resource_pack | shader | plugin,
	url: String,
	path: String,
	force: yes | no,
	append: String
)
```

 * `id`: An identifier that the user will eventually be able to use to select specific addons from a package. Should be unique and if possible should not change between versions.
 * `filename`: The name of the addon file, with the extension. The filename should be different whenever the contents of the addon are different so that mcvm knows when to update it.
 * `kind`: What type of addon this is.
 * `url` (Optional): The remote url to download the addon from.
 * `path` (Optional): The local path to link the addon from. Adding local files is a privilege that requires elevated permissions
 * `force` (Optional): Whether to force the redownload of the addon every time the package is installed. Defaults to `no`.
 * `append` (Optional): A string to add to the name of the addon, usually a version of some sort. This is needed to differentiate addon files from the same package and same version. Defaults to nothing.

Either `url` or `path` must be set, not both or neither.

# Example
Here is a simple example for a package that would install the *Sodium* mod. As this is an example, not all versions are covered.
```
@install {
	if not side client {
		finish;
	}
	if not modloader fabriclike {
		fail unsupported_modloader;
	}
	set version "unset";
	if version "1.18+" {
		set url "https://cdn.modrinth.com/data/AANobbMI/versions/mc1.18.2-0.4.1/sodium-fabric-mc1.18.2-0.4.1%2Bbuild.15.jar";
		set version "1.18";
	}
	if version "1.19+" {
		set url "https://cdn.modrinth.com/data/AANobbMI/versions/oYfJQ6lR/sodium-fabric-mc1.19.3-0.4.8%2Bbuild.22.jar";
		set version "1.19";
	}
	if value $version "unset" {
		fail unsupported_version;
	}
	addon "sodium" "Sodium.jar" (
		kind: mod,
		url: $url,
		append: $version
	);
}
```
