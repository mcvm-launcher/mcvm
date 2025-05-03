# Backup
ID: `backup`

The Backup plugin is used to automatically or manually create backups of some or all the files in an instance. Note that it isn't ideal for backing up individual worlds as it doesn't offer that much control.

## Usage

### Configuring
You need to configure backups for an instance before you will be able to make them. This specifies which files you want to include in the backup, as well as other settings. Configuration is placed in the custom config for the plugin, and looks like this:
```
{
	"custom_config": {
		"instance_id": {
			...
		}
	}
}
```
The configuration for a single instance looks like this:
```
{
	"common": {
		"paths": [string],
		"max_count": number,
		"storage_type": "folder" | "archive"
	},
	"groups: [GroupConfig]
}
```
- `common`: Common configuration for all backups for this instance.
- `common.paths`: Paths to include in the backup, relative to the `.minecraft` directory or server directory for the instance. Glob patterns are supported. By default, no files will be included.
- `common.max_count`: The maximum number of backups that can be created for whatever group. After this count is exceeded, the oldest backup will be automatically deleted. By default, an indefinite amount are allowed.
- `common.storage_type`: How the backup should be stored on the system. `archive` will use a `.zip` or `.tar.gz` file depending on your operating system. By default, `archive` is used.
- `groups`: Configuration for backup groups

#### Groups
Groups allow you to define different sets of settings for backups for the same instance, and can also automatically create backups. They have all the same fields as the `common` config, and simply override it. They also have some additional fields:
```
{
	"on": "launch" | "stop" | "interval",
	"interval": string
}
```
- `on`: When to automatically create the backup. By default, this backup group will not be created automatically. `"launch"` will create a backup whenever the game starts, and `"stop"` will create one whenever the game stops or crashes, but not when MCVM itself crashes. `"interval"` will create backups periodically as the instance is running, at whatever interval you specify in the `interval` field.
- `interval`: The interval to create periodic backups at. Ends with either `s`, `m`, `h`, or `d` for seconds, minutes, hours, and days. Example: `30s`.

### Commands
- `mcvm backup list <instance>`: List the backups for an instance
- `mcvm backup create <instance> [-g group]`: Manually create a new backup for an instance. The `-g` flag can be used to specify a group. If one isn't specified, the common settings will be used for the backup and it will not be part of any group.
- `mcvm backup info <instance> [-g group] <backup>`: Get information about a specific backup
- `mcvm backup restore <instance> [-g group] <backup>`: Restore a backup to an instance, overwriting any existing files
- `mcvm backup remove <instance> [-g group] <backup>`: Remove a backup without restoring it
