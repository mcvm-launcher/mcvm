# Custom Files
ID: `custom_files`

The Custom Files plugin allows you to easily share the same file across all instances under a profile. Right now, the plugin only supports sharing single files, not folders or globs.

## Usage

### Configuring
Configuration is placed in the `"custom_files"` object in an instance or profile, and looks like this:
```
{
	"custom_files": {
		"files": [
			{
				"source": string,
				"target": string,
				"link": bool
			},
			...
		]
	}
}
```
Here, you define the source locations and the target destinations of each of the files you want to share. The target destination is relative to the instance directory, either `.minecraft` or the server folder, and should include the full name of the target file. The `link` option allows you to hardlink the file to all of the instances instead of fully copying it.