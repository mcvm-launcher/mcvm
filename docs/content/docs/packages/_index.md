+++
title = "Packages"
sort_by = "title"
template = "section.html"
page_template = "page.html"
+++

An MCVM package is simply a file that is evaluated to install files and dependencies. They can be either declarative JSON files or custom scripts. Scripts usually follow the format of `package-id.pkg.txt`. Declarative packages should be named `package-id.json`. Package IDs may contain only letters, numbers, and hyphens (`-`). They cannot be longer than 32 characters.

# Repository

A package repository is any server that provides an `index.json` of packages for the user to source. All that is required to run a repository yourself is to make this `index.json` under `https://example.com/api/mcvm/index.json`. An index follows this format:

```
{
	"metadata": {
		"name": string,
		"description": string,
		"mcvm_version": string
	}
	"packages": {
		"package-id": {
			"url": string,
			"path": string,
			"content_type": "script" | "declarative"
		}
	}
}
```

- `metadata.name`: The display name of the repository. Not required.
- `metadata.description`: A short description of the repository. Not required.
- `metadata.mcvm_version`: The oldest MCVM version that packages included in the repository are compatible with. Used to give warnings to the user. Not required.
- `package-id`: The ID of the package.
- `url`: The URL to the package file. Unnecessary if `path` is specified.
- `path`: The path to the package file. Unnecessary if `url` is specified. On local repositories, can be either an absolute filesystem path or a path relative to where the index is. On remote repositories, can only be a relative url from where the index is.
- `content_type`: What type of package this is. Defaults to `"script"`.

## Version Patterns

Version patterns are strings that can be used to match against one or more version of something, often Minecraft. There are a couple variants:

- `single` (Example "1.19.2"): Match a single version.
- `before` (Example "1.19.2-"): Matches a version and all versions before it (inclusive).
- `after` (Example "1.19.2+"): Matches a version and all versions after it (inclusive).
- `range` (Example "1.19.1..1.20.1"): Matches versions in a range (inclusive).
- `latest` ("latest"): Matches only the latest version.
- `any` ("*"): Matches any version.

# The purpose of host ID instructions

These should be set even if the addons for the package are not downloaded from that website. These will allow MCVM to make smart decisions in the future and automatically replace files downloaded from these sites with the correct packages and prevent file duplication.
