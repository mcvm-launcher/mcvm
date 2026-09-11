# Octane
ID: `octane`

The Octane plugin is used to improve Minecraft's performance.

## Usage

### Presets
Use the `octane_preset` field in the configuration for an instance or template to choose a preset that will affect game performance. Try out a couple to see what works for your system.
- `none`: No preset, used to clear a preset that was inherited from a template
- `balanced`: Well-rounded profile that improves performance with low stutter
- `aggressive`: Best FPS boost, but with occasional FPS drops
- `smooth`: Consistent FPS with low stutter
- `ram_saver`: Reduces RAM usage for constrained systems
- `fast_launch`: Slightly faster launch time
- `modded`: For large modpacks, allocates a lot more RAM to prevent crashes
- `aikar`: Aikar's arguments for servers
- `krusic`: Krusic's arguments for servers
- `obydux`: Obydux's arguments for servers

### CPU Priority
Use the `octane_priority` field in the configuration for an instance or template to choose a preset that will affect how much CPU time the game gets on your system.
- `none`: No preset, used to clear a preset that was inherited from a template
- `high`: High priority, should probably be what you use
- `background`: Low priority, used if you want other apps on your system to be less affected by Minecraft
