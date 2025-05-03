# Packages Guide
Packages are a new concept introduced by MCVM to allow easy installation of mods, resource packs, and more. By using them, you don't have to worry about dependencies, downloading, what folders to use, or mod conflicts. Everything mostly just works.

## 1. Syncing
Before you start using packages, you need to fetch the list of available packages from the remote repositories using the `mcvm package sync` command. This also needs to be done whenever you want to use new packages that are released or new versions of those packages. This is done explicitly so that new versions of packages will never break your instances when you update them.

## 2. Finding the packages you want
Packages are referred to using their ID, which is always lowercase. To find the packages you want, use the `mcvm package browse` command to search through and get information about the packages you want to install.

## 3. Adding packages to an instance
To add a package to an instance or profile, simply edit your configuration and add the package want to the `packages` field of that instance or profile.

Example:
```
{
	"instances": {
		"example": {
			"version": "1.20.1",
			"side": "client",
			"modloader": "fabric",
			"packages": [
				"sodium",
				"create"
			]
		}
	}
}
```

## 4. Updating packages
Now that you have added a package to an instance, make sure to run `mcvm instance update <instance>` in order to actually install the package. You should also do this whenever you remove packages, or want to update them to new versions.
