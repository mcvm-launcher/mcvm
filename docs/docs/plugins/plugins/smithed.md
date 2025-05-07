# Smithed
ID: `smithed`

The Smithed plugin will download packs directly from Smithed and install them on your instances.

## Usage
Specify the packs you want in the `smithed_packs` field on an instance. You can use a version after the `@` symbol to specify a certain version of the pack.

Example:
```
{
	"smithed_packs": [
		"pack1",
		"pack2@version"
	]
}
```
