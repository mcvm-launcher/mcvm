# Multiply
ID: `multiply`

The Multiply plugin allows you to write one set of configuration and copy it to create multiple numbered instances. For example, if you run a BungeeCord proxy with 5 possible servers, you can write one config and use it to create `server-0`, `server-1`, `server-2`, `server-3`, and `server-4`.

!!!warning Note
The plugin is only supported by the CLI for now.
!!!

## Usage

Multiply is configured using it's plugin config in `plugins.json`. The configuration should look like this:

```
{
	"instances": {
		"id": {
			"count": number,
			"start": number,
			...InstanceConfig
		}
	}
}
```

- `instances`: The configuration for instances to repeat
- `id`: The template for each copy's instance ID. Must contain `$n`, which will be replaced with the index of the current instance
- `count`: The number of copies to create
- `start` (Optional): The number to start at for indexes. Defaults to zero.
- The rest of the fields are the fields for the instance config. Remember to include a side and version, and also that you can derive from templates if you want!

## Example

Using the configuration
```
{
	"instances": {
		"foo-$n": {
			"count": 4,
			"start": 1,
			"version": "latest",
			"type": "server"
		}
	}
}
```

Will create four instances, `foo-1`, `foo-2`, `foo-3`, and `foo-4`.
