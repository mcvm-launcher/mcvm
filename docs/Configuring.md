# Configuring
MCVM can be configured by editing the `mcvm.json` file in your config directory. On Linux, this directory is `${XDG_CONFIG_DIR}/mcvm/` (usually `~/.config/mcvm/`). On Windows, the config file will be in `%APPDATA%/Roaming/mcvm/`.

## Basic structure
When you first run a command that reads from the config, a default configuration file will be created. The general structure of the config file looks like this:
```json
{
	"users": {
		"user": { .. }
	},
	"default_user": ..,
	"profiles": {
		"profile": { .. }
	},
	"packages": [
		..
	],
	"instance_presets": { .. },
	"preferences": { .. }
}
```

## Users
Users are defined in the `users` object in the base of the config. User structure looks like this:
```json
"id": {
	"type": "microsoft" | "demo" | "unverified",
	"name": String,
	"uuid": String
}
```

 * `id`: The unique identifier of the user that will be referenced in commands.
 * `type`: What type of user this is. Can be any of the following:
   * `"microsoft"`: A normal Minecraft account
	* `"demo"`: An account that owns a demo of the game
	* `"unverified"`: An unverified or 'cracked` account
 * `name`: The username for this user.
 * `uuid` (Optional): The Universally Unique Identifier for this account. Some users may not use this field, but you may get a warning if you don't specify it in the config. This is to prevent username changes from invalidating your user.

There is a field called `default_user` where you should specify which user you are currently using. Otherwise, MCVM will not know which user to start the game with.

## Profiles
Profiles are listed in the same id-value format as users under the `profiles` object. They look like this:
```json
"id": {
	"version": String,
	"modloader": String,
	"plugin_loader": String,
	"instances": { .. },
	"packages": [ .. ],
	"package_stability": "stable" | "latest"
}
```

 * `version`: The Minecraft version of the profile. Can use `"latest"` or `"latest_snapshot"` as special identifiers to get the latest version.
 * `modloader` (Optional): The modloader for the profile. Can be `"vanilla"`, `"fabric"`, `"forge"`, or `"quilt"`. Defaults to `"vanilla"`.
 * `client_type` (Optional): The modification type for the client. Can be `"vanilla"`, `"fabric"`, `"forge"`, or `"quilt"`. Defaults to using the `modloader` setting.
 * `server_type` (Optional): The modification type for the server. Can be `"vanilla"`, `"paper"`, `"fabric"`, `"forge"`, or `"quilt"`. Defaults to using the `modloader` setting.
 * `instances`: The list of instances attached to this profile.
 * `packages` (Optional): The list of packages installed for this profile.
 * `stability` (Optional): Global stability setting for all packages in this profile. Defaults to `"stable"`.

## Instances
Instances are defined in the id-value format underneath the `instances` object of a profile. They have two formats:
```json
"id": "client" | "server"
```
or
```json
"id": {
	"type": "client" | "server",
	"launch": {
		"args": {
			"jvm": Array | String,
			"game": Array | String
		},
		"memory": String | {
			"init": String,
			"max": String
		},
		"java": "adoptium" | "zulu" | String,
		"preset": "akairs" | "krusic" | "obydux",
		"quick_play": {
			"type": "world" | "server" | "realm",
			"world": String,
			"server": String,
			"port": String,
			"realm": String
		}
	},
	"options": ClientOptions | ServerOptions,
	"window": {
		"resolution": {
			"width": Integer,
			"height": Integer
		}
	},
	"datapack_folder": String,
	"preset": String
}
```

The first form just has the type of the instance.

 * `type`: The type of the instance, either `"client"` or `"server"`.
 * `launch` (Optional): Options that modify the game execution.
 * `launch.args` (Optional): Custom arguments that will be passed to the Java Virtual Machine and game. Each one is optional and can either be a string of arguments separated by spaces or a list.
 * `launch.memory` (Optional): Memory sizes for the Java heap initial and maximum space. Use a string to set both (recommended), or set them individually using an object. These follow the same format as the Java arguments (e.g. `1024M` or `10G`) and should be preferred to using custom arguments as it allows MCVM to do some extra things.
 * `launch.java` (Optional): The Java installation you would like to use. Can be either `"adoptium"` or a path to a custom Java executable. Defaults to `"adoptium"`.
 * `launch.preset` (Optional): A preset that will automatically apply changes to your launch configuration to improve your experience.
   * `"none"`: The default. No changes will be applied.
   * `"aikars"`: A popular set of tuned arguments for better performance. This works better for servers that have a lot of available memory (8GB+) and is not recommended otherwise. See https://docs.papermc.io/paper/aikars-flags for more information.
	* `"krusic"`: A supposedly faster preset that uses ZGC as opposed to G1GC.
	* `"obydux"`: Performance arguments for GraalVM EE.
 * `launch.quickplay` (Optional): Specify options for the Quick Play feature, which will automatically start the client in a world, server, or realm. The `type` field selects the kind of Quick Play that you want. Use the other fields to specify which world, server, server port, and realm you want to Quick Play into when launching. Server Quick Play will work on older versions but the other two types will not.
 * `options` (Optional): Options to apply to this instance specifically. They will override global options. They are in the format of either the client or server section depending on the instance type.
 * `window.resolution` (Optional): Set a custom resolution for the game when launching. Only available for the client. Both width and height must be present if this option is used.
 * `datapack_folder` (Optional): Make mcvm install datapack type addons to this folder instead of every existing world. This provides better behavior than the default one, but requires a modification of some sort that enables global datapacks. This path is relative to the game directory of the instance (`.minecraft` or the folder where the server.properties is).
 * `preset` (Optional): A preset from the `instance_presets` field to base this instance on.

## Packages
Packages are specified globally in the `packages` list or for a specific profile in its `packages` list. It has two valid forms:
```json
"id"
```
or
```json
{
	"id": String,
	"type": String,
	"version": Integer,
	"path": String,
	"features": [],
	"permissions": "restricted" | "standard" | "elevated",
	"stability": "stable" | "latest"
}
```

In most cases the first form is all you need. If you want more control over how the package works or need to add a local package, use the second form.

 * `id`: The identifier for the package. It is very important that this field is correct for the package to work.
 * `type`: The type of the package, either a standard `"remote"` package or a `"local"` package.
 * `version` (Optional): The version number for the package. This is not needed for remote packages but *required* for local ones.
 * `path` (Optional): The path to a local package script. Only required for local packages.
 * `features` (Optional): A list of strings for package features that you would like to enable.
 * `permissions` (Optional): The amount of control you would like to give this package. Can be `"restricted"`, `"standard"`, or `"elevated"`. Packages you do not trust should be given the `"restricted"` level. Packages that you trust and want to provide access to special commands for can be given `"elevated"`. Defaults to `"standard"`.
 * `stability` (Optional): Specify whether you want this package to use development versions of addons or not. Defaults to using the `package_stability` setting from the profile.

## Preferences
In this section you can set preferences for how the whole program will work. The format looks like this, and all fields are optional:
```json
{
	"repositories": {
		"preferred": [],
		"backup": []
	},
	"caching_strategy": "none" | "lazy" | "all"
}
```

 * `repositories`: Add custom package repositories other than the default ones. Repositories in `preferred` are placed before the default ones and repositories in `backup` are placed after. A repository is specified like this:
 ```json
 {
	"id": String,
	"url": String
 }
 ```
 The URL should start with `http://` or `https://`. Port specifiers (`:123`) are allowed.
 * `caching_strategy`: What strategy to use for locally caching package scripts. `"none"` will never cache any scripts, `"lazy"` will cache only when a package is requested, and `"all"` will cache all packages whenever you run the `package sync` command. The default option is `"lazy"`.
 * `language`: Select what language to use for mcvm. Right now this does not affect the messages inside the program, but does allow packages to do things like install additional language resource packs based on your language. By default, mcvm will try to auto-detect your system language. If this fails, it will fall back to American English. Possible values are: `"afrikaans"`, `"arabic"`, `"asturian"`, `"azerbaijani"`, `"bashkir"`, `"bavarian"`, `"belarusian"`, `"bulgarian"`, `"breton"`, `"brabantian"`, `"bosnian"`, `"catalan"`, `"czech"`, `"welsh"`, `"danish"`, `"austrian_german"`, `"swiss_german"`, `"german"`, `"greek"`, `"australian_english"`, `"canadian_english"`, `"british_english"`, `"new_zealand_english"`, `"pirate_speak"`, `"upside_down"`, `"american_english"`, `"anglish"`, `"shakespearean"`, `"esperanto"`, `"argentinian_spanish"`, `"chilean_spanish"`, `"ecuadorian_spanish"`, `"european_spanish"`, `"mexican_spanish"`, `"uruguayan_spanish"`, `"venezuelan_spanish"`, `"andalusian"`, `"estonian"`, `"basque"`, `"persian"`, `"finnish"`, `"filipino"`, `"faroese"`, `"canadian_french"`, `"european_french"`, `"east_franconian"`, `"friulian"`, `"frisian"`, `"irish"`, `"scottish_gaelic"`, `"galician"`, `"hawaiian"`, `"hebrew"`, `"hindi"`, `"croatian"`, `"hungarian"`, `"armenian"`, `"indonesian"`, `"igbo"`, `"ido"`, `"icelandic"`, `"interslavic"`, `"italian"`, `"japanese"`, `"lojban"`, `"georgian"`, `"kazakh"`, `"kannada"`, `"korean"`, `"kolsch"`, `"cornish"`, `"latin"`, `"luxembourgish"`, `"limburgish"`, `"lombard"`, `"lolcat"`, `"lithuanian"`, `"latvian"`, `"classical_chinese"`, `"macedonian"`, `"mongolian"`, `"malay"`, `"maltese"`, `"nahuatl"`, `"low_german"`, `"dutch_flemish"`, `"dutch"`, `"norwegian_nynorsk"`, `"norwegian_bokmal"`, `"occitan"`, `"elfdalian"`, `"polish"`, `"brazilian_portuguese"`, `"european_portuguese"`, `"quenya"`, `"romanian"`, `"russian_pre_revolutionary"`, `"russian"`, `"rusyn"`, `"northern_sami"`, `"slovak"`, `"slovenian"`, `"somali"`, `"albanian"`, `"serbian"`, `"swedish"`, `"upper_saxon_german"`, `"silesian"`, `"tamil"`, `"thai"`, `"tagalog"`, `"klingon"`, `"toki_pona"`, `"turkish"`, `"tatar"`, `"ukrainian"`, `"valencian"`, `"venetian"`, `"vietnamese"`, `"yiddish"`, `"yoruba"`, `"chinese_simplified"`, `"chinese_traditional_hong_kong"`, `"chinese_traditional_taiwan"`, `"malay_jawi"`.
 