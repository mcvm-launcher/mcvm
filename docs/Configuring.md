# Configuring

MCVM can be configured by editing the `mcvm.json` file in your config directory. On Linux, this directory is `${XDG_CONFIG_DIR}/mcvm/` (usually `~/.config/mcvm/`). On Windows, the config file will be in `%APPDATA%/Roaming/mcvm/`. Note that these paths are only relevant for the official CLI, as any implementation can (and should) change these directories to whatever they want.

## Basic structure

When you first run a command that reads from the config, a default configuration file will be created. The general structure of the config file looks like this:

```
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

- `packages`: Packages that will be applied to every profile. Can be overridden by profiles.

## Users

Users are defined in the `users` object in the base of the config. User structure looks like this:

```
"id": {
	"type": "microsoft" | "demo" | "unverified",
	"name": string,
	"uuid": string
}
```

- `id`: The unique identifier of the user that will be referenced in commands.
- `type`: What type of user this is. Can be any of the following:
  - `"microsoft"`: A normal Minecraft account
  - `"demo"`: An account that owns a demo of the game
  - `"unverified"`: An unverified or 'cracked` account
- `name`: The username for this user.
- `uuid` (Optional): The Universally Unique Identifier for this account. Some users may not use this field, but you may get a warning if you don't specify it in the config. This is to prevent username changes from invalidating your user.

There is a field called `default_user` where you should specify which user you are currently using. Otherwise, MCVM will not know which user to start the game with.

## Profiles

Profiles are listed in the same id-value format as users under the `profiles` object. They look like this:

```
"id": {
	"version": string,
	"modloader": modloader,
	"client_type": client_type,
	"server_type": client_type,
	"instances": { .. },
	"packages": [ .. ] | {
		"global": [ .. ],
		"client": [ .. ],
		"server": [ .. ]
	},
	"package_stability": "stable" | "latest"
}
```

- `version`: The Minecraft version of the profile. Can use `"latest"` or `"latest_snapshot"` as special identifiers to get the latest version.
- `modloader` (Optional): The modloader for the profile.
- `client_type` (Optional): The modification type for the client. Defaults to using the `modloader` setting.
- `server_type` (Optional): The modification type for the server. Defaults to using the `modloader` setting.
- `instances`: The list of instances attached to this profile.
- `packages` (Optional): Can either be a list of packages to apply to every instance in the profile, or an object of multiple lists with a different set of packages for each type of instance. The `global` key will apply to every instance. These override packages installed globally, but can be overridden by instances.
- `stability` (Optional): Global stability setting for all packages in this profile. Defaults to `"stable"`.

## Instances

Instances are defined in the id-value format underneath the `instances` object of a profile. They have two formats:

```json
"id": "client" | "server"
```

or

```
"id": {
	"type": "client" | "server",
	"launch": {
		"args": {
			"jvm": [string] | string,
			"game": [string] | string
		},
		"memory": string | {
			"init": string,
			"max": string
		},
		"env": { .. },
		"wrapper": {
			"cmd": string,
			"args": [string]
		},
		"java": "adoptium" | "zulu" | string,
		"preset": "akairs" | "krusic" | "obydux",
		"quick_play": {
			"type": "world" | "server" | "realm",
			"world": string,
			"server": string,
			"port": string,
			"realm": string
		},
		"use_log4j_config": bool
	},
	"options": ClientOptions | ServerOptions,
	"window": {
		"resolution": {
			"width": integer,
			"height": integer
		}
	},
	"datapack_folder": string,
	"snapshots": {
		"paths": [string],
		"max_count": integer,
		"storage_type": "folder" | "archive
	},
	"packages": [ .. ],
	"preset": string
}
```

The first form just has the type of the instance. All fields are optional unless stated otherwise.

- `type` (Required): The type of the instance, either `"client"` or `"server"`.
- `launch`: Options that modify the game execution.
- `launch.args`: Custom arguments that will be passed to the Java Virtual Machine and game. Each one is optional and can either be a string of arguments separated by spaces or a list.
- `launch.memory`: Memory sizes for the Java heap initial and maximum space. Use a string to set both (recommended), or set them individually using an object. These follow the same format as the Java arguments (e.g. `1024M` or `10G`) and should be preferred to using custom arguments as it allows MCVM to do some extra things.
- `launch.env`: A map of strings to strings that let you set environment variables for the game program.
- `launch.wrapper`: A command to wrap the launch command in. Set the command and its arguments.
- `launch.java`: The Java installation you would like to use. Can either be one of `"auto"`, `"system"`, `"adoptium"`, or `"zulu"`, or a path to a custom Java installation. Defaults to `"auto"`, which automatically picks or downloads the best Java flavor for your system. The `"system"` setting will try to find an existing installation on your system, and will fail if it doesn't find one. If the system setting doesn't find Java even though you know it is installed, let us know with an issue. The custom Java path must have the jvm executable at `{path}/bin/java`.
- `launch.preset`: A preset that will automatically apply changes to your launch configuration to improve your experience.
  - `"none"`: The default. No changes will be applied.
  - `"aikars"`: A popular set of tuned arguments for better performance. This works better for servers that have a lot of available memory (8GB+) and is not recommended otherwise. See https://docs.papermc.io/paper/aikars-flags for more information.				self.set_index(&mut cursor).context("Failed to set index")?;
- `launch.use_log4j_config`: Whether to use Mojang's config for Log4J on the client. Defaults to false.

- `datapack_folder`: Make MCVM install datapack type addons to this folder instead of every existing world. This provides better behavior than the default one, but requires a modification of some sort that enables global datapacks. This path is relative to the game directory of the instance (`.minecraft` or the folder where the server.properties is).
- `snapshots`: Options for snapshots, which allow you to create backups of the files in an instance.
- `snapshots.paths`: The relative paths from the instance directory to store when making snapshots. By default, no files will be backed up.
- `snapshots.max_count`: The maximum number of snapshots to keep before automatically deleting the oldest ones. By default, there is no limit.
- `snapshots.storage_type`: What format snapshots should be stored in. Defaults to `"archive"`.
- `packages`: Packages to install on this instance specifically. Overrides packages installed globally and on the profile.
- `preset`: A preset from the `instance_presets` field to base this instance on.

## Packages

Packages are specified globally in the `packages` list or for a specific profile in its `packages` list. It has two valid forms:

```
"id"
```

or

```
{
	"id": string,
	"type": string,
	"features": [string],
	"use_default_features": bool,
	"permissions": "restricted" | "standard" | "elevated",
	"stability": "stable" | "latest",
	"worlds": [string]
}
```

In most cases the first form is all you need. If you want more control over how the package works or need to add a local package, use the second form.

- `id`: The identifier for the package. It is very important that this field is correct for the package to work.
- `type`: The type of the package, either a standard `"repository"` package or a `"local"` package.
- `features` (Optional): A list of strings for package features that you would like to enable.
- `use_default_features` (Optional): Whether or not to use the default features of this package. `true` by default.
- `permissions` (Optional): The amount of control you would like to give this package. Can be `"restricted"`, `"standard"`, or `"elevated"`. Packages you do not trust should be given the `"restricted"` level. Packages that you trust and want to provide access to special commands for can be given `"elevated"`. Defaults to `"standard"`.
- `stability` (Optional): Specify whether you want this package to use development versions of addons or not. Defaults to using the `package_stability` setting from the profile.
- `worlds` (Optional): A list of worlds to only apply addons like datapacks to. If left empty (the default), will apply to all worlds in the instance.

## Preferences

In this section you can set preferences for how the whole program will work. The format looks like this, and all fields are optional:

```
{
	"repositories": {
		"preferred": [],
		"backup": [],
		"enable_core": boolean,
		"enable_std": boolean
	},
	"package_caching_strategy": "none" | "lazy" | "all"
}
```

- `repositories`: Add custom package repositories other than the default ones. Repositories in `preferred` are placed before the default ones and repositories in `backup` are placed after. A repository is specified like this:

```
{
	"id": string,
	"url": string,
	"path": string
}
```

Either `url` or `path` must be set. `path` allows you to have repository indices on your local machine.
The URL should start with `http://` or `https://`. Port specifiers (`:123`) are allowed.

- `repositories.enable_core`: Whether to enable the internal package repository. Defaults to true.
- `repositories.enable_std`: Whether to enable the standard package repository. Defaults to true.
- `package_caching_strategy`: What strategy to use for locally caching package scripts. `"none"` will never cache any scripts, `"lazy"` will cache only when a package is requested, and `"all"` will cache all packages whenever you run the `package sync` command. The default option is `"lazy"`.
- `language`: Select what language to use for MCVM. Right now this does not affect the messages inside the program, but does allow packages to do things like install additional language resource packs based on your language. By default, MCVM will try to auto-detect your system language. If this fails, it will fall back to American English. Possible values are: `"afrikaans"`, `"arabic"`, `"asturian"`, `"azerbaijani"`, `"bashkir"`, `"bavarian"`, `"belarusian"`, `"bulgarian"`, `"breton"`, `"brabantian"`, `"bosnian"`, `"catalan"`, `"czech"`, `"welsh"`, `"danish"`, `"austrian_german"`, `"swiss_german"`, `"german"`, `"greek"`, `"australian_english"`, `"canadian_english"`, `"british_english"`, `"new_zealand_english"`, `"pirate_speak"`, `"upside_down"`, `"american_english"`, `"anglish"`, `"shakespearean"`, `"esperanto"`, `"argentinian_spanish"`, `"chilean_spanish"`, `"ecuadorian_spanish"`, `"european_spanish"`, `"mexican_spanish"`, `"uruguayan_spanish"`, `"venezuelan_spanish"`, `"andalusian"`, `"estonian"`, `"basque"`, `"persian"`, `"finnish"`, `"filipino"`, `"faroese"`, `"canadian_french"`, `"european_french"`, `"east_franconian"`, `"friulian"`, `"frisian"`, `"irish"`, `"scottish_gaelic"`, `"galician"`, `"hawaiian"`, `"hebrew"`, `"hindi"`, `"croatian"`, `"hungarian"`, `"armenian"`, `"indonesian"`, `"igbo"`, `"ido"`, `"icelandic"`, `"interslavic"`, `"italian"`, `"japanese"`, `"lojban"`, `"georgian"`, `"kazakh"`, `"kannada"`, `"korean"`, `"kolsch"`, `"cornish"`, `"latin"`, `"luxembourgish"`, `"limburgish"`, `"lombard"`, `"lolcat"`, `"lithuanian"`, `"latvian"`, `"classical_chinese"`, `"macedonian"`, `"mongolian"`, `"malay"`, `"maltese"`, `"nahuatl"`, `"low_german"`, `"dutch_flemish"`, `"dutch"`, `"norwegian_nynorsk"`, `"norwegian_bokmal"`, `"occitan"`, `"elfdalian"`, `"polish"`, `"brazilian_portuguese"`, `"european_portuguese"`, `"quenya"`, `"romanian"`, `"russian_pre_revolutionary"`, `"russian"`, `"rusyn"`, `"northern_sami"`, `"slovak"`, `"slovenian"`, `"somali"`, `"albanian"`, `"serbian"`, `"swedish"`, `"upper_saxon_german"`, `"silesian"`, `"tamil"`, `"thai"`, `"tagalog"`, `"klingon"`, `"toki_pona"`, `"turkish"`, `"tatar"`, `"ukrainian"`, `"valencian"`, `"venetian"`, `"vietnamese"`, `"yiddish"`, `"yoruba"`, `"chinese_simplified"`, `"chinese_traditional_hong_kong"`, `"chinese_traditional_taiwan"`, `"malay_jawi"`.
