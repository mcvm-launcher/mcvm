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
Instructions are individual commands that are run inside routines for your package script. Instructions are separated by semicolons. They often have arguments that can either be an identifier or a string.

### Variables
Any instruction arguments that take a string can also take a variable, with the syntax `$variable_name`. You can also use string substitution to combine multiple variables, with the syntax `"Hello ${variable}!"`. This syntax can be escaped in the string using a backslash. Using a variable that is not defined directly will cause the routine to fail. Using a variable that is not defined in a substitution string will fill it with an empty string.

### Routine Context
Most instructions can only be run in certain routines or in routines called by those specific routines.

Logic, relationships with other packages, and addons can only be used in the `@install` context.

Metadata like `description` and `authors` can only be used in the `@meta` context.

### List of Instructions
 * `if {condition} [arguments...] { ... }`: If instructions let you run instructions inside a block only if a condition is met at runtime. The valid conditions are:
	 * `value {x} {y}`: Check if two strings are the same. This is meant to be used to check the value of variables.
	 * `version {pattern}`: Check that the Minecraft version of this instance matches a pattern.
	 * `modloader {vanilla | fabric | forge | quilt | fabriclike}`: Checks if the modloader supports a mod type. The `fabriclike` option will match both Fabric and Quilt and should be used for most Fabric mods unless you know they don't play nice with Quilt.
	 * `plugin_loader {vanilla | bukkit}`: Checks if the plugin loader supports a plugin type.
	 * `side {client | server}`: Check what instance type the package is being installed on.
	 * `feature {name}`: Check if a feature is enabled for this package.
	 * `os {windows | linux}`: Check if the user is using a certain operating system.
	 * `defined {variable_name}`: Check if a variable has been defined.
	 * `stability {stable | latest}`: Check for the configured stability of the package. You should check this and only install release versions of addons if `stable` is selected.
	 * `language {language}`: Check the user's configured language.
	 * `not {condition}`: Inverts a condition. You can chain these, but why would you want to.
	 * `and {left} {right}`: Checks if both conditions are true.
	 * `or {left} {right}`: Checks if either one of the conditions are true.
 * `set {variable} {value}`: Sets the value of a variable.
 * `finish`: Will silently end the routine.
 * `fail [unsupported_version | unsupported_modloader | unsupported_plugin_loader]`: End execution with an error.
 * `addon {id} {filename} (..)`: Add an addon to the instance. This is the main goal of a package. The name field is the filename of the addon. Keys and values are put inside the parentheses.
 * `require {package1} {package2} ...`: Create a dependency on one or more packages. Use this for libraries that your package depends on. Check the core packages folder to see some standard packages that you can require.
 * `refuse {package}`: Specifies that this package is incompatible with another. These packages will be unable to coexist together. Both packages do not need to refuse each other, just one refuse instruction in one package will suffice.
 * `bundle {package}`: Bundle another package with this one. Useful for packages that group together multiple other packages, such as modpacks. Prefer using this over `require` when you aren't including a library as it has a different semantic meaning to mcvm.
 * `recommend {package}`: Recommend to the user that they should use another package if it is not installed.
 * `compat {package} {compat_package}`: Automatically install `compat_package` if `package` is present.
 * `extend {package}`: Specify that this package extends the functionality of another. An error will be created if the package is not present.
 * `notice {message}`: Display a warning or important information as a message to the user. Notice messages may not be more than 128 characters long, and there cannot be more than five of them that are displayed per package evaluation.
 * `name {name}`: Set the display name of the package.
 * `description {description}`: Set the description for this package.
 * `version {version}`: Set the version of this package. This has no actual meaning to mcvm and should be used only for project versions.
 * `authors {author1} {author2} ...`: Set a list of authors for this package. This should be the authors of the project itself, not the package script.
 * `package_maintainers {author1} {author2} ...`: Set a list of maintainers for this package. This should be the maintainers of the package script, not the project itself.
 * `website {website}`: Set a primary website / repository link / project link / etc.
 * `support_link {link}`: Set a support / donation link.

### The `addon` Instruction
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
 * `version` (Optional): The version of this addon. This is important because it lets mcvm differentiate between different versions of the file for caching purposes. If this field is not present, the addon will never be cached and will be redownloaded every time.

Either `url` or `path` must be set, not both or neither.

### The `require` Instruction
The require instruction has a syntax of a list of package groups, which can either be multiple strings inside parentheses or a single string. In the future, these groups will be able to be chained in more complex expressions, but for now they have no purpose. Just put the packages in a list.

Another part is the ability to make an explicit dependency using the `<"package-name">` syntax (Note that the brackets are outside of the string). This allows you to depend on another package, but not install it automatically. The user must specify that they want the package manually. Use this for packages that you require as a dependency, but may have gameplay changes or side effects that you want the user to be aware of.

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
	if version "1.18+" {
		set url "https://cdn.modrinth.com/data/AANobbMI/versions/mc1.18.2-0.4.1/sodium-fabric-mc1.18.2-0.4.1%2Bbuild.15.jar";
		set version "1.18";
	}
	if version "1.19+" {
		set url "https://cdn.modrinth.com/data/AANobbMI/versions/oYfJQ6lR/sodium-fabric-mc1.19.3-0.4.8%2Bbuild.22.jar";
		set version "1.19";
	}
	if not defined $version {
		fail unsupported_version;
	}
	addon "sodium" "Sodium.jar" (
		kind: mod,
		url: $url,
		version: $version
	);
}
```
