# Plugin User Guide

Plugins are extensions to MCVM that add new functionality. They can add new subcommands, support extra modloaders, and more. They allow MCVM to be modular and only include the features you need, keeping it less bloated and making it easier to maintain.

## ⚠️WARNING⚠️

Plugins work as arbitrary programs that run on your system. Malicious plugins can gain unauthorized access to your computer, steal personal information or account details, or damage files. Protect yourself by only downloading verified plugins from the `plugin install` command, and never interact with files from someone you don't trust.

## Installing

### Using The Installer

Use the `mcvm plugin browse` command to see a list of available plugins. Then, you can run `mcvm plugin install {plugin}` to install the plugin you want.

### Manually

If you have plugin files you are sure you can trust, first locate the `plugins` directory under your MCVM data directory. If the plugin is one file with the `.json` extension, you can simply move it to that folder. If it is a `.zip` file, extract the file into the `plugins` directory, ensuring that there is a directory named after the plugin and it has a file named `plugin.json` directly inside, and not under any subfolders after that.

### Enabling The Plugin

Whichever installation method you use, you must now enable the plugin in your configuration. MCVM should have created a file in your config directory named `plugins.json`. You can edit it with the `mcvm config edit-plugins` command. Add the name of the plugin you just installed like so:

```json
{
	"plugins": ["plugin_name"]
}
```

Now, run the `mcvm plugin list` command to see the list of plugins, and the one you just added should be in the list and marked as "Loaded". You can also use the `mcvm plugin enable` and `mcvm plugin disable` commands to enable and disable plugins.

## Configuring

Plugins can be configured to change their behavior. Most of their configuration is specific to the plugin, and you will have to check with their documentation to see how it is formatted. To configure a plugin, simply add an entry under the `config` field in your plugins config like so:

```json
{
	"plugins": [
		"plugin_name"
	],
	"config": {
		"plugin_name": {
			...
		}
	}
}
```
