Package scripts can be more difficult to maintain for most packages which just need to install one or more addons. When you need more control over the install process, a package script can be used.

# Syntax

At the root level, a package is organized into **routines** which describe a list of instructions to be run to perform some action. Routines can have any name, but some have special meaning.

```
@routine_name {
	...
}
```

The main routine that will be in every single package is the `@install` routine. This routine is run when the package is installed or updated in order to download files for your game.
The `@meta` routine contains instructions that set optional metadata for the package such as display name, license, authors, etc.
The `@properties` routine can be used to set certain properties for the package, such as default features.

## Instructions

Instructions are individual commands that are run inside routines for your package script. Instructions are separated by semicolons. They often have arguments that can either be an identifier or a string.

### Variables

Any instruction arguments that take a string can also take a variable, with the syntax `$variable_name`. You can also use string substitution to combine multiple variables, with the syntax `"Hello ${variable}!"`. This syntax can be escaped in the string using a backslash. Using a variable that is not defined directly will cause the routine to fail. Using a variable that is not defined in a substitution string will fill it with an empty string.

#### Special Constants

Certain special variables will be defined when you run your script. These cannot be modified by scripts. Not all implementations will define all variables. If one is marked as optional, you should check that it is defined before using it.

- `$MINECRAFT_VERSION`: The Minecraft version you are installing for.

### Routine Context

Most instructions can only be run in certain routines or in routines called by those specific routines.

Logic, relationships with other packages, and addons can only be used in the `@install` context.

Metadata like `description` and `authors` can only be used in the `@meta` context.

Properties like `features` and `modrinth_id` can only be used in the `@properties` context.

### List of Instructions

#### Installation Instructions

- `if {condition} [arguments...] { ... }`: If instructions let you run instructions inside a block only if a condition is met at runtime. The valid conditions are:
  - `value {x} {y}`: Check if two strings are the same. This is meant to be used to check the value of variables.
  - `version {pattern}`: Check that the Minecraft version of this instance matches a pattern.
  - `modloader {vanilla | fabric | forge | quilt | fabriclike}`: Checks if the modloader supports a mod type. The `fabriclike` option will match both Fabric and Quilt and should be used for most Fabric mods unless you know they don't play nice with Quilt.
  - `plugin_loader {vanilla | bukkit}`: Checks if the plugin loader supports a plugin type.
  - `side {client | server}`: Check what instance type the package is being installed on.
  - `feature {name}`: Check if a feature is enabled for this package.
  - `os {windows | mac | linux}`: Check if the user is using a certain operating system.
  - `defined {variable_name}`: Check if a variable has been defined.
  - `stability {stable | latest}`: Check for the configured stability of the package. You should check this and only install release versions of addons if `stable` is selected.
  - `language {language}`: Check the user's configured language.
  - `not {condition}`: Inverts a condition. You can chain these, but why would you want to.
  - `and {left} {right}`: Checks if both conditions are true.
  - `or {left} {right}`: Checks if either one of the conditions are true.
- `set {variable} {value}`: Sets the value of a variable.
- `finish`: Will silently end the routine.
- `fail [unsupported_version | unsupported_modloader | unsupported_plugin_loader | unsupported_features | unsupported_operating_system]`: End execution with an error.
- `call {routine}`: Runs the contents of another routine. The called routine cannot be reserved by MCVM. Possibly recursive structures are also not allowed. MCVM will reject them.
- `addon {id} [filename] (..)`: Add an addon to the instance. Keys and values are put inside the parentheses.
- `require {package1} {package2} ...`: Create a dependency on one or more packages.
- `refuse {package}`: Specifies that this package is incompatible with another.
- `bundle {package}`: Bundle another package with this one.
- `recommend {package}`: Recommend to the user that they should use another package if it is not installed. Putting an exclamation point before the package string (e.g. `recommend !"pkg";`) will invert the recommendation.
- `compat {package} {compat_package}`: Make a compat with other packages.
- `extend {package}`: Extend another package.
- `notice {message}`: Display a warning or important information as a message to the user. Notice messages may not be more than 128 characters long, and there cannot be more than five of them that are displayed per package evaluation.
- `cmd {command} {arg1} {arg2} ...`: Run a command on the system. Requires elevated permissions. Only runs during the install stage, not when resolving dependencies. If the command returns a non-zero exit code, the install process will fail. Context such as current working directory is not persisted across commands.

#### Metadata Instructions

- `name {name}`: Set the display name of the package.
- `description {description}`: Set a short description for this package.
- `long_description {description}`: Set a long description for this package.
- `version {version}`: Set the display version of this package.
- `authors {author1} {author2} ...`: Set the list of authors for this package.
- `package_maintainers {author1} {author2} ...`: Set the list of maintainers for this package.
- `website {website}`: Set a primary website / project link / etc.
- `support_link {link}`: Set a support / donation link.
- `documentation {link}`: Set a wiki / documentation link.
- `source {link}`: Set a source / repository link.
- `issues {link}`: Set an issue tracker link.
- `community {link}`: Set a Discord / forum link.
- `icon {link}`: Set a link to a small square icon image.
- `banner {link}`: Set a link to a large background / banner image.
- `license {license}`: Set the project license.

#### Properties Instructions

- `features {feature1} {feature2} ...`: Set the allowed features for this package.
- `default_features {feature1} {feature2} ...`: Set the features enabled by default for this package.
- `modrinth_id {id}`: Set the Modrinth ID.
- `curseforge_id {id}`: Set the CurseForge ID.
- `supported_modloaders`: Set the supported modloaders.
- `supported_plugin_loaders`: Set the supported plugin loaders.
- `supported_sides`: Set the supported sides.

### The `addon` Instruction

The `addon` instruction is a bit more complex. Inside the parentheses you put a set of keys and values to configure the addon and how it is installed. The full addon config looks like this:

```
addon id filename (
	kind: mod | resource_pack | shader | plugin,
	url: String,
	path: String,
	version: String,
	hash_sha256: String,
	hash_sha512: String
)
```

Either `url` or `path` must be set, not both or neither.

### The `require` Instruction

The require instruction has a syntax of a list of package groups, which can either be multiple strings inside parentheses or a single string. In the future, these groups will be able to be chained in more complex expressions, but for now they have no purpose. Just put the packages in a list.

Another part is the ability to make an explicit dependency using the `<"package-name">` syntax (Note that the brackets are outside of the string).

# Example

Here is a simple example for a package that would install the _Sodium_ mod. As this is an example, not all versions are covered.

```
@meta {
	name "Sodium";
}
@properties {
	modrinth_id "AANobbMI";
}
@install {
	if not side client {
		finish;
	}
	if not modloader fabriclike {
		fail unsupported_modloader;
	}
	if version "1.18" {
		set url "https://cdn.modrinth.com/data/AANobbMI/versions/mc1.18.2-0.4.1/sodium-fabric-mc1.18.2-0.4.1%2Bbuild.15.jar";
		set version "74Y5Z8fo";
	}
	if version "1.19" {
		set url "https://cdn.modrinth.com/data/AANobbMI/versions/oYfJQ6lR/sodium-fabric-mc1.19.3-0.4.8%2Bbuild.22.jar";
		set version "oYfJQ6lR";
	}
	if not defined version {
		fail unsupported_version;
	}
	addon "mod" (
		kind: mod,
		url: $url,
		version: $version
	);
}
```
