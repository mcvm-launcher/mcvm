+++
title = "Stats"
+++
ID: `stats`

The Stats plugin collects playtime stats while instances are running and then can report them. Note that these stats are for your information only and are never sent anywhere.

## Usage
Playtime stats will be collected for every instance. Simply run `mcvm stats` to view them.

### Configuration
Configuration is done in the custom config for the plugin.
```
"stats": {
	"use_live_tracking": bool
}
```
- `use_live_tracking`: Whether or not to update playtime stats every minute while an instance is running. This will make playtime stats more resilient to crashes, but also use more processing for lots of instances. Defaults to `false`.
