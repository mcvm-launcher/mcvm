+++
title = "Declarative Packages"
+++

This is the format for declarative packages

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
	"gallery": [string],
	"license": string,
	"keywords": [string],
	"categories": [string]
}
```

- `name`: Display name of the package.
- `description`: A short description of the package. Should be 1-2 sentences max.
- `long_description`: A longer description of the package.
- `authors`: A list of authors for this package. This should be the authors of the project / addons themselves, not the MCVM package file.
- `package_maintainers`: A list of maintainers for this package. This should be the maintainers of the MCVM package file, not the project itself
- `website`: Primary website / project link / etc.
- `support_link`: Support / donation / sponsored link.
- `documentation`: Wiki / documentation link.
- `source`: Source / repository link.
- `issues`: Issue tracker link.
- `community`: Discord / chat / forum link.
- `icon`: Link to a small square icon image.
- `banner`: Link to a large background / banner image.
- `gallery`: Links to gallery images.
- `license`: The project license. Should be the short / abbreviated version. If a longer license is needed, provide a link to the license file in this field.
- `keywords`: Search term keywords for this package. Keep them short and sweet.
- `categories`: Categories your package is in, such as library or adventure.

## Properties

Properties for the package that do have a meaning to MCVM and other package hosts / installers. All fields are optional.

```
{
	"features": [string],
	"default_features": [string],
	"modrinth_id": string,
	"curseforge_id": string,
	"supported_versions": [VersionPattern],
	"supported_modloaders": ["vanilla" | "fabric" | "forge" | "quilt" | "fabriclike"],
	"supported_plugin_loaders": ["vanilla" | "bukkit"],
	"supported_sides": ["client" | "server"],
	"supported_operating_systems": ["windows" | "linux" | "macos" | "unix" | "other"],
	"supported_architectures": ["x86" | "x86_64" | "arm" | "other"],
	"tags": [string],
	"open_source": bool
}
```

- `features`: A list of available features for this package. Features can be enabled or disabled by the user to configure how the package is installed.
- `default_features`: The features that will be enabled by default.
- `modrinth_id`: ID of the project for this package on Modrinth, if applicable. See [the purpose of host ID instructions](Packages.md#the-purpose-of-host-id-instructions).
- `curseforge_id`: ID of the project for this package on CurseForge, if applicable. See [the purpose of host ID instructions](Packages.md#the-purpose-of-host-id-instructions).
- `smithed_id`: ID of the project for this package on Smithed, if applicable. See [the purpose of host ID instructions](Packages.md#the-purpose-of-host-id-instructions).
- `supported_versions`: Minecraft versions supported by this package. Defaults to all of them.
- `supported_modloaders`: Modloaders supported by this package. Defaults to all of them.
- `supported_plugin_loaders`: Plugin loaders supported by this package. Defaults to all of them.
- `supported_sides`: Game sides supported by this package. Defaults to both of them.
- `supported_operating_systems`: Operating systems supported by this package. Defaults to all of them.
- `supported_architectures`: System architectures supported by this package. Defaults to all of them.
- `tags`: Similar to categories and keywords, but with actual meaning. Packages will be able to use tags in the future to depend on any package with a tag, or refuse it.
- `open_source`: Say if this package is open source. If this property is not set, the open source status will be inferred from the license string.

## Relations

Relations are dependencies / conflicts / etc. with other packages. All fields are optional unless stated otherwise.

```
{
	"dependencies": [string],
	"explicit_dependencies": [string],
	"conflicts": [string],
	"extensions": [string],
	"bundled": [string],
	"compats": [[string, string]],
	"recommendations": [{
		"value" (Required): string,
		"invert": bool
	}]
}
```

- `dependencies`: Library packages that your package depends on. Check the core packages folder to see some standard packages that you can require.
- `explicit_dependencies`: The same as dependencies. However, these libraries also change the behavior of the game enough that it would be good for the user to know about them. These packages must be required by the user in their config as well.
- `conflicts`: Packages that this package is incompatible with.
- `extensions`: Packages that this package extends the functionality of. For example, if this package was an addon mod for the Create mod, then it would extend the `create` package. Will cause an error if the other package does not exist.
- `bundled`: Packages included with this one. Useful for packages that group together multiple other packages, such as modpacks. Prefer using this over `dependencies` when you aren't including a library as it has a different semantic meaning to MCVM.
- `compats`: A list of lists with two values, a source package and destination package. If the source package exists, the destination package will be automatically installed.
- `recommendations`: Packages that will be recommended to the user if they are not installed. `value` is the package to be recommended. Setting `invert` to true will instead recommend _against_ the use of the package.

## Conditions

Condition sets are used in multiple parts of declarative packages to check properties of the installation environment. All fields are optional, and will not contribute to the condition if left empty.

```
{
	"minecraft_versions": [VersionPattern],
	"side": "client" | "server",
	"modloaders": [modloader_match],
	"plugin_loaders": [plugin_loader_match],
	"stability": "stable" | "latest",
	"features": [string],
	"content_versions": [string],
	"operating_systems": [operating_system],
	"architectures": [architecture],
	"languages": [Language]
}
```

- `minecraft_versions`: Check if any one of the version patterns in the list matches the used Minecraft version.
- `side`: Check whether this package is being installed on client or server.
- `modloaders`: Check if the users's modloader matches any of the `modloader_match`'s.
- `plugin_loaders`: Check if the users's plugin loader matches any of the `plugin_loader_match`'s.
- `stability`: Check for the configured stability of the package.
- `features`: Check if all of the listed features are enabled for this package.
- `content_versions`: Check if the user has configured any of the given content versions for this package.
- `operating_systems`: Check the operating system this package is being installed on.
- `architectures`: Check the system architecture this package is being installed on.
- `languages`: Check the user's configured language matches one of the listed ones.

## Addons

Addons are the actual files that are installed to a user's game. They are specified in a map.

```
{
	"addon-id": {
		"kind": "mod" | "resource_pack" | "shader" | "plugin",
		"versions": [
			...
		],
		"conditions": [ConditionSet],
		"optional": boolean
	}
}
```

- `addon-id`: The ID of the addon. This lets MCVM differentiate between addons from the same package and allows the user to modify specific addons from a package. Thus, try not to change it between updates of your package.
- `kind`: What type of addon / modification this is.
- `versions`: A list of versions for this addon. See the addon versions section.
- `conditions` (Optional): A list of conditions for the installation of this addon. If any of these conditions fails, the addon will not be installed, but no errors will be shown. Thus, it is better to use the `supported_...` properties for this purpose.
- `optional` (Optional): Whether this addon should be considered optional when evaluating the package. If this is set to false, and no versions of the addon are matched when evaluating, then the evaluation will fail. Defaults to false.

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

- ConditionSet: Addon versions contain all the fields of a ConditionSet. These conditions are used to filter down and find the version that satisfies all the requirements. If multiple versions satisfy the requirements, the evaluator will first favor versions with a content version that is newer. Then, it will favor versions that are more specific to your system ("fabric" modloader over "fabriclike", for example). Finally, the one that comes first in the list is chosen.
- `url`: A URL to the file for this version. Not required if `path` is specified.
- `path`: A local filesystem path to the addon file. Not required if `url` is specified. Requires elevated permissions.
- `version` (Optional): The unique version identifier of this addon. This is important because it lets MCVM differentiate between different versions of the file for caching purposes. If this field is not present, the addon will never be cached and will be redownloaded every time. This ID should not contain any special characters.
- `filename` (Optional): The name of the addon file in the instance, with the extension. This is not required and is not recommended most of the time as MCVM will already generate a unique filename for it that does not clash with other files.
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
		"relations": Relations,
		"notices": [string]
	}
}
```

- `conditions`: A list of condition sets to check.
- `properties`: The changes to apply if the conditions are satisfied.
- `properties.relations` (Optional): Package relations to include. These are appended to the other relations
- `properties.notices` (Optional): A list of messages to display to the user.
