# Plugin Hook Protocol
This page describes all of the mechanisms available to a plugin to communicate with MCVM. You can use it to help you write simple scripts or make your own plugin framework libraries for different languages.

## Environment Variables
MCVM sets multiple environment variables on the running plugin executable

- `MCVM_PLUGIN`: Always set whenever running as a plugin. Can be used to make sure that the executable is being run by MCVM and not a user by accident.
- `MCVM_CUSTOM_CONFIG`: Custom configuration for this plugin in the `plugins.json` file. In a JSON format.
- `MCVM_DATA_DIR`: Path to MCVM's data directory
- `MCVM_CONFIG_DIR`: Path to MCVM's config directory
- `MCVM_PLUGIN_STATE`: The current value of this plugin's persistent state, sent as JSON
- `MCVM_VERSION`: The version of MCVM that is running the plugin
- `HOOK_VERSION`: The version of the hook that is running. Can be used to prevent sending back invalid data.
- `PLUGIN_LIST`: The list of all enabled plugins, separated by commas. Will include the plugin that is running as well.

## Arguments
Arguments to the executable will always be passed in this order
1. Additional arguments as specified in the hook handler in the plugin manifest
2. The ID of the hook that is being run
3. The argument to the hook

## Output
This section is not applicable if using a hook that takes over, such as the `subcommand` hook. For most hooks, output is read line-by-line, with each line starting with the delimiter `%_` to separate output commands from stray prints. Following this delimiter, each line must then have a JSON item of one of the following types. *However*, this JSON is also base64-encoded by default to prevent special characters like line breaks in strings from ruining the output. If you do not wish to have this behavior, `raw_transfer` can be enabled in the plugin manifest. Remember to not pretty-print your JSON and keep it in one line to prevent errors.

- `text`: Displays text to the output
```
{
	"text": [string, "important" | "extra" | "debug" | "trace"]
}
```
- `message`: Displays a message to the output
```
{
	"message": {
		"contents": MessageContents,
		"level": "important" | "extra" | "debug" | "trace"
	}
}
```
- `start_process`: Starts an output process
```
"start_process"
```
- `end_process`: Ends an output process
```
"end_process"
```
- `start_section`: Starts an output section
```
"start_section"
```
- `end_section`: Ends an output section
```
"end_section"
```
- `set_result`: Sets the result / output of the hook to serialized JSON. This must be the last thing you output, as after this the plugin runner will stop listening to the plugin and move on.
```
{
	"set_result": string
}
```
- `set_state`: Sets the persistent state of the plugin
```
{
	"set_state": any
}
