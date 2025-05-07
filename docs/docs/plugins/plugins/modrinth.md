# Modrinth
ID: `modrinth`

The Modrinth plugin will download projects directly from Modrinth and install them on your instances.

## Usage
Specify the projects you want in the `modrinth_projects` field on an instance. You can use a version after the `@` symbol to specify a certain version of the project. The version must be the version ID, not the version name.

Example:
```
{
	"modrinth_projects": [
		"project1",
		"project2@version"
	]
}
```
