# Packages

An MCVM package is simply a file that is evaluated to install files and dependencies. They can be either declarative JSON files or custom scripts. Scripts usually follow the format of `package-id.pkg.txt`. Declarative packages should be named `package-id.json`. Package IDs may contain only letters, numbers, and hyphens (`-`). They cannot be longer than 32 characters.

# Repository

A package repository is any server that provides an `index.json` of packages for the user to source. All that is required to run a repository yourself is to make this `index.json` under `https://example.com/api/mcvm/index.json`. An index follows this format:

```
{
	"packages": {
		"package-id": {
			"version": Integer,
			"url": string,
			"content_type": "script" | "declarative"
		}
	}
}
```

- `package-id`: The ID of the package.
- `version`: The package version known by the repository.
- `url`: The URL to the `package.pkg.txt` file.
- `content_type`: What type of package this is. Defaults to `"script"`.

# Declarative format

```
{
	"meta": Metadata,
	"properties": Properties,
	"relations": Relations,
	"addons": {
		...
	},
	"conditional_rules": [
		...
	]
}
```

- `meta`: Set package metadata. See the metadata section.
- `properties`: Set package properties. See the properties section.
- `relations`: Specify relationships with other packages. See the relations section.
- `addons`: Install addons using this package. See the addons section.
- `conditional_rules`: Apply changes to this packages depending on conditions. See the conditional rules section.

## Metadata

Metadata for a package is extra information about a package such as its display name and authors. All fields are optional.

```
{
	"name": string,
	"description": string,
	"long_description": string,
	"version": string,
	"authors": [string],
	"package_maintainers": [string],
	"website": string,
	"support_link": string,
	"documentation": string,
	"source": string,
	"issues": string,
	"community": string,
	"icon": string,
	"banner": string,
	"license": string
}
```

- `name`: Display name of the package.
- `description`: A short description of the package. Should be 1-2 sentences max.
- `long_description`: A longer description of the package.
- `version`: A project version. Has no meaning.
- `authors`: A list of authors for this package. This should be the authors of the project / addons itself, not the mcvm package file.
- `package_maintainers`: A list of maintainers for this package. This should be the maintainers of the mcvm package file, not the project itself
- `website`: Primary website / project link / etc.
- `support_link`: Support / donation / sponsored link.
- `documentation`: Wiki / documentation link.
- `source`: Source / repository link.
- `issues`: Issue tracker link.
- `community`: Discord / chat / forum link.
- `icon`: Link to a small square icon image.
- `banner`: Link to a large background / banner image.
- `license`: The project license. Should be the short / abbreviated version. If a longer license is needed, provide a link to the license file in this field.

## Properties

Properties for the package that do have a meaning to mcvm and other package hosts / installers. All fields are optional.

```
{
	"features": [string],
	"default_features": [string],
	"modrinth_id": string,
	"curseforge_id": string,
	"supported_modloaders": ["vanilla" | "fabric" | "forge" | "quilt" | "fabriclike"],
	"supported_plugin_loaders": ["vanilla" | "bukkit"],
	"supported_sides": ["client" | "server"]
}
```

- `features`: A list of available features for this package. Features can be enabled or disabled by the user to configure how the package is installed.
- `default_features`: The features that will be enabled by default.
- `modrinth_id`: ID of the project for this package on Modrinth, if applicable. See "The purpose of host ID instructions".
- `curseforge_id`: ID of the project for this package on CurseForge, if applicable. See "The purpose of host ID instructions".
- `supported_modloaders`: Modloaders supported by this package. Defaults to all of them.
- `supported_plugin_loaders`: Plugin loaders supported by this package. Defaults to all of them.
- `supported_sides`: Game sides supported by this package. Defaults to both of them.

## Relations

Relations are dependencies / conflicts / etc. with other packages. All fields are optional.

```
{
	"dependencies": [string],
	"explicit_dependencies": [string],
	"conflicts": [string],
	"extensions": [string],
	"bundled": [string],
	"compats": [[string, string]],
	"recommendations": [string]
}
```

- `dependencies`: Library packages that your package depends on. Check the core packages folder to see some standard packages that you can require.
- `explicit_dependencies`: The same as dependencies. However, these libraries also change the behavior of the game enough that it would be good for the user to know about them. These packages must be required by the user in their config as well.
- `conflicts`: Packages that this package is incompatible with.
- `extensions`: Packages that this package extends the functionality of. For example, if this package was an addon mod for the Create mod, then it would extend the `create` package. Will cause an error if the other package does not exist.
- `bundled`: Packages included with this one. Useful for packages that group together multiple other packages, such as modpacks. Prefer using this over `dependencies` when you aren't including a library as it has a different semantic meaning to mcvm.
- `compats`: A list of lists with two values, a source package and destination package. If the source package exists, the destination package will be automatically installed.
- `recommendations`: Packages that will be recommended to the user if they are not installed.

## Version Patterns

Version patterns are strings that can be used to match against one or more version of something, often Minecraft. There are a couple variants:

- `single` (Example "1.19.2"): Match a single version.
- `before` (Example "1.19.2-"): Matches a version and all versions before it (inclusive).
- `after` (Example "1.19.2+"): Matches a version and all versions after it (inclusive).
- `range` (Example "1.19.1..1.20.1"): Matches versions in a range (inclusive).
- `latest` ("latest"): Matches only the latest version.
- `any` ("*"): Matches any version.

## Conditions

Condition sets are used in multiple parts of declarative packages to check properties of the installation environment. All fields are optional, and will not contribute to the condition if left empty.

```
{
	"minecraft_versions": [VersionPattern],
	"side": "client" | "server",
	"modloaders": [string],
	"plugin_loaders": [string],
	"stability": "stable" | "latest",
	"features": [string],
	"os": "windows" | "mac" | "linux",
	"language": Language
}
```

- `minecraft_versions`: Check if any one of the version patterns in the list matches the used Minecraft version.
- `side`: Check whether this package is being installed on client or server.
- `modloaders`: Check if any of these modloaders matches the users's modloader. Same options as the `supported_modloaders` property.
- `plugin_loaders`: Check if any of these plugin loaders matches the user's plugin loader. Same options as the `supported_plugin_loaders` property.
- `stability`: Check for the configured stability of the package.
- `features`: Check if all of the listed features are enabled for this package.
- `os`: Check the operating system this package is being installed on.
- `language`: Check the user's configured language.

## Addons

Addons are the actual files that are installed to a user's game. They are specified in a map.

```
{
	"addon-id": {
		"kind": "mod" | "resource_pack" | "shader" | "plugin",
		"versions": [
			...
		],
		"conditions": [ConditionSet]
	}
}
```

- `addon-id`: The ID of the addon. This lets mcvm differentiate between addons from the same package and allows the user to modify specific addons from a package. Thus, try not to change it between updates of your package.
- `kind`: What type of addon / modification this is.
- `versions`: A list of versions for this addon. See the addon versions section.
- `conditions` (Optional): A list of conditions for the installation of this addon. If any of these conditions fails, the addon will not be installed, but no errors will be shown. Thus, it is better to use the `supported_...` properties for this purpose.

## Addon Versions

Addon versions are different files and versions of an addon

```
{
	ConditionSet...,
	"url": string,
	"path": string,
	"version": string,
	"relations": Relations,
	"filename": string,
	"notices": [string],
	"hashes": {
		"sha256": string,
		"sha512": string
	}
}
```

- ConditionSet: Addon versions contain all the fields of a ConditionSet. These conditions are used to filter down and find the version that satisfies all the requirements. If multiple versions satisfy the requirements, the one that comes first in the list is chosen.
- `url`: A URL to the file for this version. Not required if `path` is specified.
- `path`: A local filesystem path to the addon file. Not required if `url` is specified. Requires elevated permissions.
- `version` (Optional): The unique version identifier of this addon. This is important because it lets mcvm differentiate between different versions of the file for caching purposes. If this field is not present, the addon will never be cached and will be redownloaded every time. This ID should not contain any special characters.
- `filename` (Optional): The name of the addon file in the instance, with the extension. This is not required and is not recommended most of the time as mcvm will already generate a unique filename for it that does not clash with other files.
- `relations` (Optional): Extra package relations to apply if this addon version is chosen.
- `notices` (Optional): A list of messages to display to the user if this version is chosen.
- `hashes` (Optional): Different fields for hashes of this version file. Allows MCVM to check for valid files when downloading them.

Either `url` or `path` must be set, not both or neither.

## Conditional Rules

Conditional rules let you change the package based on ConditionSets. Each rule will apply the properties only if all of the conditions are satisfied.

```
{
	"conditions": [ConditionSet],
	"properties": {
		"relations": Relations
	}
}
```

- `conditions`: A list of condition sets to check.
- `properties`: The changes to apply if the conditions are satisfied.
- `properties.relations`: Package relations to include. These are appended to the other relations
- `properties.notices`: A list of messages to display to the user.

# The purpose of host ID instructions

These should be set even if the addons for the package are not downloaded from that website. These will allow mcvm to make smart decisions in the future and automatically replace files downloaded from these sites with the correct packages and prevent file duplication.
