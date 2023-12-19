# Modifications

MCVM and its packages support multiple different types of modifications to the game, such as modloaders and plugin types. The package format attempts to support as many modifications as possible, but that does not mean that every launcher is able to install all of them automatically.

The different types of fields are listed here. Variants may be listed with `supported` or `unsupported` depending on whether MCVM supports installing them.

## Client types (`client_type`)

- `vanilla`: The standard game. (supported)
- `fabric`: The Fabric modloader. (supported)
- `quilt`: The Quilt modloader. (supported)
- `forge`: The MinecraftForge modloader. (unsupported)
- `neoforged`: The NeoForged modloader. (unsupported)
- `liteloader`: The LiteLoader modloader. (unsupported)
- `risugamis`: Risugami's modloader. (unsupported)
- `rift`: The Rift modloader. (unsupported)

## Server types (`server_type`)

- `vanilla` The standard game. (supported)
- `paper` Papermc server (supported)
- `sponge` SpongeVanilla server (unsupported)
- `spongeforge` SpongeForge server (unsupported)
- `craftbukkit` CraftBukkit server (unsupported)
- `spigot` Spigot server (unsupported)
- `glowstone` Glowstone server (unsupported)
- `pufferfish` Pufferfish server (unsupported)
- `purpur` Purpur server (unsupported)
- `folia` Folia server (supported)
- `fabric` The Fabric modloader. (supported)
- `quilt` The Quilt modloader. (supported)
- `forge` The Forge modloader. (unsupported)
- `neoforged` The NeoForged modloader. (unsupported)
- `risugamis` Risugami's modloader. (unsupported)
- `rift` The Rift modloader. (unsupported)

## Modloaders (`modloader`)

Setting a modloader is an easy way to set the same client type and server type on a profile. This includes any modloading game types that are included on both client and server.

- `vanilla` (supported)
- `fabric` (supported)
- `quilt` (supported)
- `forge` (unsupported)
- `neoforged` (unsupported)
- `risugamis` (unsupported)
- `rift` (unsupported)

## Modloader matches (`modloader_match`)

Modloader matches are used in packages to match different client and server types that support a mod format

- `vanilla`
- `fabric`
- `quilt`
- `forge`
- `neoforged`
- `liteloader`
- `risugamis`
- `rift`
- `fabriclike`: Matches any loader that supports loading Fabric mods (Fabric and Quilt).
- `forgelike`: Matches any loader that supports loading Forge mods (MinecraftForge, NeoForged, and SpongeForge).

## Plugin loader matches (`plugin_loader_match`)

Plugin loader matches are used in packages to match different server types that support a plugin format

- `vanilla`
- `paper`
- `sponge`
- `craftbukkit`
- `spigot`
- `glowstone`
- `pufferfish`
- `purpur`
- `folia`
- `bukkit`: Matches any server that can load Bukkit plugins (CraftBukkit, Paper, Spigot, Glowstone, Pufferfish, and Purpur).
