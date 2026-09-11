# Cleanup
ID: `cleanup`

The Cleanup plugin adds utilities to get rid of old files.

!!!warning Note
Cleanup can only be run from the CLI for now.
!!!

## Usage
### Commands
- `nitro cleanup version <version>`: Remove assets for a Minecraft version that aren't used by other versions
- `nitro cleanup addons`: Remove cached versions of addons that aren't needed anymore
- `nitro cleanup fabric-cache`: Remove processed versions of mods for each Fabric instance which will be rebuilt when the instance is launched again
- `nitro cleanup skins`: Remove cached skin files from other players. Usually small, but can be large if you play in public servers a lot.