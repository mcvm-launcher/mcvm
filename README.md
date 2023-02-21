mcvm is a lightweight Minecraft launcher meant to provide a better interface with custom content such as mods and resource packs.

In the current launcher, you have to waste time moving files in and out of folders, downloading mods from sketchy websites, setting up servers, and sharing your config to play with friends. mcvm hopes to alleviate some of these pains with its smart versioning system.

*Profiles* in mcvm are groups of settings, modloaders, and the such with a game version. They are shared config for versions of the game which you then base *instances* off of. Instances are programs such as a client or server which have a parent instance and are the thing you actually run.

*Packages* are the big selling point of mcvm. They are scripts which are installed on a profile and run so that they obtain the correct content files for your game. The `sodium` package, for example, when installed on a profile, will add the Sodium jar file to your mods folder only on client instances.

Right now, the launcher is in an early state. Packages are not complete, you cannot log in to the game in online mode, and generally a lot of features are missing. If you see something you want that isn't there, try contributing!
