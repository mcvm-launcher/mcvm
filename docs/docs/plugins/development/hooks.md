# Hooks

Hooks are what give your plugin functionality. They are essentially custom event handlers that can be written in any language. They can run code whenever something happens, or inject new items into some of the data-driven parts of MCVM. Handlers for hooks are defined in the plugin manifest.

## Parts of a Hook
- ID: Every hook has a unique ID used to identify it
- Argument and Result: These are the inputs and outputs of the hook. They can be any JSON type, such as a string or object, and depend on which hook you are handling.

## How Hooks are Run
Most of the time when MCVM calls a hook, it will check every plugin that supports that hook, and call the hook on each one to create a final list of results. Handlers are not exclusive; multiple plugins can subscribe to the same hook. However, some hooks are only called on specific plugins. For example, the `on_load` hook is only called on a specific plugin once it is loaded.

## List of Hooks

### `on_load`
Called when this plugin is loaded. Can be used to set up state and such.
- Argument: None
- Result: None

### `subcommand`
Called whenever one of the subcommands that this hook registers are run. The argument is the list of arguments that were provided to the subcommand, *including* the subcommand itself. Note that this hook also takes over output, meaning anything coming from stdout will be output to the console instead.
- Argument: `[string]`
- Result: None

### `modify_instance_config`
Called on every instance to possibly modify its config. The output config will be merged with the instance's current config in the same way as profiles are. Note that the input is not sequential: All plugins will be given the same config before modification, instead of applying one after the other, and the results will all be merged together.
- Argument:
```
{
	"config": InstanceConfig
}
```
- Result:
```
{
	"config": InstanceConfig
}
```

### `add_versions`
This hook allows you to add extra Minecraft versions to the version manifest, allowing them to be specified in instance configuration and automatically downloaded.
- Argument: None
- Result:
```
[
	{
		"id": string,
		"type": "release" | "snapshot" | "old_alpha" | "old_beta",
		"url": string,
		"is_zipped": bool
	},
	...
]
```

### `on_instance_setup`
Called when an instance is being set up, for update or launch. Can return modifications to make to the launch parameters
resulting from installing a certain modification
- Argument:
```
{
	"id": string,
	"side": "client" | "server",
	"game_dir": string,
	"version_info": {
		"version": string,
		"versions": [string]
	},
	"client_type": ClientType,
	"server_type": ServerType,
	"custom_config": {...},
	"internal_dir": string,
	"update_depth": "shallow" | "full" | "force"
}
```
- Result:
```
{
	"main_class_override": string | null,
	"jar_path_override": string | null,
	"classpath_extension": [string]
}
```

### `remove_game_modification`
Called when the game modifications (client or server type) of an instance change, to allow cleaning up old or invalid files. Will be given the client / server type that needs to be removed.
- Argument:
```
{
	"id": string,
	"side": "client" | "server",
	"game_dir": string,
	"version_info": {
		"version": string,
		"versions": [string]
	},
	"client_type": ClientType,
	"server_type": ServerType,
	"custom_config": {...},
	"internal_dir": string,
	"update_depth": "shallow" | "full" | "force"
}
```
- Result: None

### `on_instance_launch`
Called whenever an instance is launched
- Argument: InstanceLaunchArg
- Result: None

### `while_instance_launch`
Also called when an instance is launched, but is non-blocking, and runs alongside the instance. Can be used for periodic tasks and such.
- Argument: InstanceLaunchArg
- Result: None

### `on_instance_stop`
Called when an instance is stopped. This happens when Minecraft is closed or crashes. This hook will *not* be called if MCVM crashes while the instance is running.
- Argument: InstanceLaunchArg
- Result: None

### `custom_package_instruction`
Handles custom instructions in script packages.
- Argument:
```
{
	"pkg_id": string,
	"command": string,
	"args": [string]
}
```
- Result:
```
{
	"handled": bool,
	"addon_reqs": [
		{
			"id": string,
			"file_name": string | null,
			"kind": "resource_pack" | "mod" | "plugin" | "shader" | "datapack",
			"url": string | null,
			"path": string | null,
			"version": string | null,
			"hashes": {
				"sha256": string | null,
				"sha512": string | null
			}
		}
	],
	"deps": [
		{
			"value": string,
			"explicit": bool
		}
	],
	"conflicts": [string],
	"recommendations": [
		{
			"value": string,
			"invert": bool
		}
	],
	"bundled": [string],
	"compats": [[string, string]],
	"extensions": [string],
	"notices": [string]
}
```
- `handled`: Whether this instruction was handled or not. Should be false if this instruction is not for your plugin.

## `handle_auth`
Handles authentication with custom user types
- Argument:
```
{
	"user_id": string,
	"user_type": string
}
```
- Result:
```
{
	"handled": bool,
	"profile": {
		"name": string,
		"id": string,
		"skins": [
			{
				"id": string,
				"url" string,
				"state": "active" | "inactive",
				"variant": "classic" | "slim"
			}
		],
		"capes": [
			{
				"id": string,
				"url" string,
				"state": "active" | "inactive",
				"alias": string
			}
		]
	} | null
}
```
- `profile.id`: The UUID of the user

### `add_translations`
Adds extra translations to MCVM
- Argument: None
- Result:
```
{
	"language": {
		"key": "translation",
		...
	},
	...
}
```

### `add_instance_transfer_formats`
Adds information about new transfer formats that this plugin adds support for. Returns a list of formats, including information about features that they support and don't support.
- Argument: None
- Result:
```
[
	{
		"id": string,
		"import": {
			"modloader": "supported" | "format_unsupported" | "plugin_unsupported",
			"mods": "supported" | "format_unsupported" | "plugin_unsupported",
			"launch_settings": "supported" | "format_unsupported" | "plugin_unsupported"
		} | null,
		"export": {
			"modloader": "supported" | "format_unsupported" | "plugin_unsupported",
			"mods": "supported" | "format_unsupported" | "plugin_unsupported",
			"launch_settings": "supported" | "format_unsupported" | "plugin_unsupported"
		} | null
	},
	...
]
```

### `export_instance`
Hook called on a specific plugin to export an instance using one of the formats it supports
- Argument:
```
{
	"format": string,
	"id": string,
	"name": string,
	"side": "client" | "server",
	"game_dir": string,
	"result_path": string,
	"minecraft_version": string | "latest" | "latest_snapshot",
	"client_type": ClientType,
	"server_type": ServerType
}
```
- `id`: The instance ID
- `result_path`: The desired path to the output file
- Result: None

### `import_instance`
Hook called on a specific plugin to import an instance using one of the formats it supports
- Argument:
```
{
	"format": string,
	"id": string,
	"source_path": string,
	"result_path": string
}
```
- `id`: The desired ID of the resulting instance
- `source_path`: The path to the instance to import
- `result_path`: Where to place the files for the imported instance
- Result:
```
{
	"format": string,
	"name": string | null,
	"side": "client" | "server",
	"version": string | "latest" | "latest_snapshot",
	"client_type": ClientType,
	"server_type": ServerType
}
```

### `add_supported_game_modifications`
Adds extra game modifications to the list of supported ones for installation. This should be done
if you plan to install these game modifications with your plugin.
- Argument: None
- Result:
```
{
	"client_types": [ClientType],
	"server_types": [ServerType]
}
```

### `add_instances`
Adds new instances to the config
- Argument: None
- Result:
```
[
	InstanceConfig,
	InstanceConfig,
	...
]
```

## Common Types
### InstanceLaunchArg
```
{
	"id": string,
	"side": "client" | "server",
	"dir": string,
	"game_dir": string,
	"version_info": {
		"version": string,
		"versions": [string]
	},
	"custom_config": {...},
	"pid": integer
}
```
