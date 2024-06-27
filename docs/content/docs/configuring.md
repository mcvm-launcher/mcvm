+++
title = "Configuring"
+++

MCVM can be configured by editing the `mcvm.json` file in your config directory. On Linux, this directory is `${XDG_CONFIG_DIR}/mcvm/` (usually `~/.config/mcvm/`). On Windows, the config file will be in `%APPDATA%/Roaming/mcvm/`. Note that these paths are only relevant for the official CLI, as any implementation can (and should) change these directories to whatever they want.

## Basic structure

When you first run a command that reads from the config, a default configuration file will be created. The general structure of the config file looks like this:

```
{
	"users": {
		"user": { .. }
	},
	"default_user": ..,
	"instances": {
		"instance": { .. }
	},
	"profiles": {
		"profile": { .. }
	},
	"global_profile": { .. },
	"instance_groups": {
		"group": [ .. ]
	},
	"preferences": { .. }
}
```

- `global_profile`: An optional global profile that all other profiles will inherit from
- `instance_groups`: Named groups of instance IDs that can be used to easily refer to multiple instances

## Users

Users are defined in the `users` object in the base of the config. User structure looks like this:

```
"id": {
	"type": "microsoft" | "demo"
}
```

- `id`: The unique identifier of the user that will be referenced in commands.
- `type`: What type of user this is. Can be any of the following:
  - `"microsoft"`: A normal Minecraft account
  - `"demo"`: An account that owns a demo of the game

There is a field called `default_user` where you should specify which user you are currently using. Otherwise, MCVM will not know which user to start the game with by default and you will have to specify it every time.

## Instances

Instances are defined in the id-value format underneath the `instances` object of the config. They look like this:

```
"id": {
	"type": "client" | "server",
	"from": string | [string],
	"version": string,
	"name": string,
	"modloader": modloader,
	"client_type": client_type,
	"server_type": client_type,
	"package_stability": "stable" | "latest",
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
		"java": "auto" | "system" | "adoptium" | "zulu" | "graalvm" | string,
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
	"packages": [ .. ],
	"preset": string
}
```

The first form just has the type of the instance. All fields are optional unless stated otherwise.

- `type` (Required): The type of the instance, either `"client"` or `"server"`.
- `from`: A [profile](#profiles) or multiple profiles to derive configuration from. The config from each profile will be applied in order, and then the config for this instance will be applied last.
- `version`: The Minecraft version of the instance. Can use `"latest"` or `"latest_snapshot"` as special identifiers to get the latest version. This is technically a required field, but can be derived from a profile instead.
- `name`: A custom display name for this instance. Has no rules and does not have to be unique.
- `modloader`: The modloader for the instance, which can be used to set both the client and server type automatically.
- `client_type`: The modification type for the client. Defaults to using the `modloader` setting.
- `server_type`: The modification type for the server. Defaults to using the `modloader` setting.
- `package_stability`: Global stability setting for all packages in this instance. Defaults to `"stable"`.
- `launch`: Options that modify the game execution.
- `launch.args`: Custom arguments that will be passed to the Java Virtual Machine and game. Each one is optional and can either be a string of arguments separated by spaces or a list.
- `launch.memory`: Memory sizes for the Java heap initial and maximum space. Use a string to set both (recommended), or set them individually using an object. These follow the same format as the Java arguments (e.g. `1024M` or `10G`) and should be preferred to using custom arguments as it allows MCVM to do some extra things.
- `launch.env`: A map of strings to strings that let you set environment variables for the game program.
- `launch.wrapper`: A command to wrap the launch command in. Set the command and its arguments.
- `launch.java`: The Java installation you would like to use. Can either be one of `"auto"`, `"system"`, `"adoptium"`, `"zulu"`, or `"graalvm"`, or a path to a custom Java installation. Defaults to `"auto"`, which automatically picks or downloads the best Java flavor for your system. The `"system"` setting will try to find an existing installation on your system, and will fail if it doesn't find one. If the system setting doesn't find Java even though you know it is installed, let us know with an issue. The custom Java path must have the JVM executable at `{path}/bin/java`.
- `launch.use_log4j_config`: Whether to use Mojang's config for Log4J on the client. Defaults to false.
- `datapack_folder`: Make MCVM install datapack type addons to this folder instead of every existing world. This provides better behavior than the default one, but requires a modification of some sort that enables global datapacks. This path is relative to the game directory of the instance (`.minecraft` or the folder where the server.properties is).
- `packages`: Packages to install on this instance specifically. Overrides packages installed on the profile.
- `preset`: A preset from the `instance_presets` field to base this instance on.

## Profiles

Profiles allow you to easily share configuration between instances and keep them in sync without having to rewrite the same thing many times. Instances and profiles can use the `from` field to derive from other profiles in a composable manner. Profiles are listed in the same id-value format as instances under the `profiles` object. They look like this:

```
"id": {
	InstanceConfig...,
	"packages": [ .. ] | {
		"global": [ .. ],
		"client": [ .. ],
		"server": [ .. ]
	}
}
```

- `InstanceConfig`: Profiles have all of the same fields as instances, which they provide to instances that derive them
- `packages` (Optional): Can either be a list of packages to apply to every instance in the profile, or an object of multiple lists with a different set of packages for each type of instance. The `global` key will apply to every instance.

## Packages

Packages are specified in an instance's package list or for a profile in its packages list. Each package has two valid forms:

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

In most cases the first form is all you need. If you want more control over how the package works, use the second form.

- `id`: The identifier for the package. It is very important that this field is correct for the package to work.
- `type`: The type of the package, currently only allowing a standard `"repository"` package.
- `features` (Optional): A list of strings for package features that you would like to enable.
- `use_default_features` (Optional): Whether or not to use the default features of this package. `true` by default.
- `permissions` (Optional): The amount of control you would like to give this package. Can be `"restricted"`, `"standard"`, or `"elevated"`. Packages you do not trust should be given the `"restricted"` level. Packages that you trust and want to provide access to special commands for can be given `"elevated"`. Defaults to `"standard"`.
- `stability` (Optional): Specify whether you want this package to use development versions of addons or not. Defaults to using the `package_stability` setting from the profile.
- `worlds` (Optional): A list of worlds to only apply addons like datapacks to. If left empty (the default), will apply to all worlds in the instance.

## Plugins

Plugins are configured in a separate file called `plugins.json` in the same directory as your normal config file.

```
{
	"plugins": [
  	"plugin_name" | {
  		"name": string,
  		"config": any
  	}
  	...
  ]
}
```

The `plugins` field allows you to specify a list of enabled plugins and options you want for them

- `plugin_name`: The name / ID of the plugin to enable
- `config` (Optional): Custom configuration to give to the plugin. This will differ for whatever plugin you are using, and some do not need it at all.

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
	"package_caching_strategy": "none" | "lazy" | "all",
	"language": language
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
The URL should start with `http://` or `https://`. Port specifiers (`:123`) are allowed. You can also use sub-paths of a URL like `https://example.com/foo` to use multiple repositories from the same site.

- `repositories.enable_core`: Whether to enable the internal package repository. Defaults to true.
- `repositories.enable_std`: Whether to enable the standard package repository. Defaults to true.
- `package_caching_strategy`: What strategy to use for locally caching package scripts. `"none"` will never cache any scripts, `"lazy"` will cache only when a package is requested, and `"all"` will cache all packages whenever you run the `package sync` command. The default option is `"all"`.
- `language`: Select what language to use for MCVM. This will affect translations for many messages if you have a translation plugin installed, and also allows packages to do things like install additional language resource packs based on your language. By default, MCVM will try to auto-detect your system language. If this fails, it will fall back to American English. Possible values are: `"afrikaans"`, `"arabic"`, `"asturian"`, `"azerbaijani"`, `"bashkir"`, `"bavarian"`, `"belarusian"`, `"bulgarian"`, `"breton"`, `"brabantian"`, `"bosnian"`, `"catalan"`, `"czech"`, `"welsh"`, `"danish"`, `"austrian_german"`, `"swiss_german"`, `"german"`, `"greek"`, `"australian_english"`, `"canadian_english"`, `"british_english"`, `"new_zealand_english"`, `"pirate_speak"`, `"upside_down"`, `"american_english"`, `"anglish"`, `"shakespearean"`, `"esperanto"`, `"argentinian_spanish"`, `"chilean_spanish"`, `"ecuadorian_spanish"`, `"european_spanish"`, `"mexican_spanish"`, `"uruguayan_spanish"`, `"venezuelan_spanish"`, `"andalusian"`, `"estonian"`, `"basque"`, `"persian"`, `"finnish"`, `"filipino"`, `"faroese"`, `"canadian_french"`, `"european_french"`, `"east_franconian"`, `"friulian"`, `"frisian"`, `"irish"`, `"scottish_gaelic"`, `"galician"`, `"hawaiian"`, `"hebrew"`, `"hindi"`, `"croatian"`, `"hungarian"`, `"armenian"`, `"indonesian"`, `"igbo"`, `"ido"`, `"icelandic"`, `"interslavic"`, `"italian"`, `"japanese"`, `"lojban"`, `"georgian"`, `"kazakh"`, `"kannada"`, `"korean"`, `"kolsch"`, `"cornish"`, `"latin"`, `"luxembourgish"`, `"limburgish"`, `"lombard"`, `"lolcat"`, `"lithuanian"`, `"latvian"`, `"classical_chinese"`, `"macedonian"`, `"mongolian"`, `"malay"`, `"maltese"`, `"nahuatl"`, `"low_german"`, `"dutch_flemish"`, `"dutch"`, `"norwegian_nynorsk"`, `"norwegian_bokmal"`, `"occitan"`, `"elfdalian"`, `"polish"`, `"brazilian_portuguese"`, `"european_portuguese"`, `"quenya"`, `"romanian"`, `"russian_pre_revolutionary"`, `"russian"`, `"rusyn"`, `"northern_sami"`, `"slovak"`, `"slovenian"`, `"somali"`, `"albanian"`, `"serbian"`, `"swedish"`, `"upper_saxon_german"`, `"silesian"`, `"tamil"`, `"thai"`, `"tagalog"`, `"klingon"`, `"toki_pona"`, `"turkish"`, `"tatar"`, `"ukrainian"`, `"valencian"`, `"venetian"`, `"vietnamese"`, `"yiddish"`, `"yoruba"`, `"chinese_simplified"`, `"chinese_traditional_hong_kong"`, `"chinese_traditional_taiwan"`, `"malay_jawi"`.
