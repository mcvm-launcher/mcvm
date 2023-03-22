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
	"preferences": { .. }
}
```

## Users
Users are defined in the `users` object in the base of the config. User structure looks like this:
```json
"id": {
	"type": String,
	"name": String,
	"uuid": String
}
```

 * `id`: The unique identifier of the user that will be referenced in commands.
 * `type`: What type of user this is. Can either be `"microsoft"` for a normal Minecraft account or `"demo"` for a demo account.
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
	"packages": [ .. ]
}
```

 * `version`: The Minecraft version of the profile.
 * `modloader` (Optional): The modloader for the profile. Can be `"vanilla"`, `"fabric"`, `"forge"`, or `"quilt"`. Defaults to `"vanilla"`.
 * `plugin_loader` (Optional): The server plugin loader for the profile. Can be `"vanilla"` or `"paper"`. Defaults to `"vanilla"`.
 * `instances`: The list of instances attached to this profile.
 * `packages` (Optional): The list of packages installed for this profile.

## Instances
Instances are defined in the id-value format underneath the `instances` object of a profile. They look like this:
```json
"id": {
	"type": String,
	"launch": {
		"args": {
			"jvm": Array | String,
			"game": Array | String
		},
		"memory": String | {
			"init": String,
			"max": String
		},
		"java": String,
		"preset": String
	}
}
```

 * `type`: The type of the instance, either `"client"` or `"server"`.
 * `launch` (Optional): Options that modify the game execution.
 * `launch.args` (Optional): Custom arguments that will be passed to the Java Virtual Machine and game. Each one is optional and can either be a string of arguments separated by spaces or a list.
 * `launch.memory` (Optional): Memory sizes for the Java heap initial and maximum space. Use a string to set both (recommended), or set them individually using an object. These follow the same format as the Java arguments (e.g. `1024M` or `10G`) and should be preferred to using custom arguments as it allows MCVM to do some extra things.
 * `launch.java` (Optional): The Java installation you would like to use. Can be either `"adoptium"` or a path to a custom Java executable. Defaults to `"adoptium"`.
 * `launch.preset` (Optional): A preset that will automatically apply changes to your launch configuration to improve your experience.
   * `"none"`: The default. No changes will be applied.
   * `"aikars"`: A popular set of tuned arguments for better performance. This works better for servers that have a lot of available memory (8GB+) and is not recommended otherwise. See https://docs.papermc.io/paper/aikars-flags for more information.

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
	"version": String,
	"path": String,
	"features": [],
	"permissions": String
}
```

In most cases the first form is all you need. If you want more control over how the package works or need to add a local package, use the second form.

 * `id`: The identifier for the package. It is very important that this field is correct for the package to work.
 * `type`: The type of the package, either a standard `"remote"` package or a `"local"` package.
 * `version` (Optional): The version string for the package. This is not needed for remote packages but *required* for local ones.
 * `path` (Optional): The path to a local package script. Only required for local packages.
 * `features` (Optional): A list of strings for package features that you would like to enable.
 * `permissions` (Optional): The amount of control you would like to give this package. Can be `"restricted"`, `"standard"`, or `"elevated"`. Packages you do not trust should be given the `"restricted"` level. Packages that you trust and want to provide access to special commands for can be given `"elevated"`. Defaults to `"standard"`.

## Preferences
In this section you can set preferences for how the whole program will work. The format looks like this, and all fields are optional:
```json
{
	"repositories": {
		"preferred": [],
		"backup": []
	}
}
```

 * `repositories`: Add custom package repositories other than the default ones. Repositories in `preferred` are placed before the default ones and repositories in `backup` are placed after. A repository is specified like this:
 ```json
 {
	"id": String,
	"url": String
 }
 ```