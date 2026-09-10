# Datapacks

Datapacks can be installed on Nitrolaunch instances just like any other package. However, they also have some extra configuration that can be used to customize them.

You will get the best datapack experience by installing from Smithed, as it will bundle both the datapack and it's resource pack together in one package.

## Custom Datapack Folder

By default, Nitrolaunch will install datapack packages to every existing world. It will also check for new worlds being created while the game is running. However, this system is not perfect, so it is usually better to use a global datapacks mod like Paxy or Global Datapacks if you can afford not being in vanilla.

!!!warning Note
The default datapack propagation can only apply datapacks after the world has been created, so you will need to reload any new worlds for datapacks to apply. This means any datapacks required for world generation will not work with this system, and you should probably use global datapacks.
!!!

+++ App
Not implemented yet.
+++ CLI
Configure the `datpack_folder` field on the instance using `nitro instance edit` to be the relative path to the datapack folder.
+++

## Smithed Base Template

If you want to make your life a lot easier, the Smithed plugin provides you with the `smithed-base` template which you can inherit on your instances. It has a global datapack mod and the datapack folder configuration already set up, as well as some mods to fix some datapack bugs.

## Using Specific Worlds

You can also configure specific packages to only install a datapack on a specific set of worlds.

+++ App
Not implemented yet.
+++ CLI
Using the expanded package configuration syntax (check the [configuration reference](../reference/configuring.md) for more info), set the `worlds` field on the package to the list of world names you want to install the datapack in.
+++

## Welding Datapacks

The [Weld](../plugins/plugins/weld.md) plugin can be used to enhance compatibility with lots of datapacks.