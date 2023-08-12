# MCVM

MCVM is a lightweight Minecraft launcher meant to provide a better interface with custom content such as mods and resource packs.

In the official launcher, you have to waste time moving files in and out of folders, downloading mods from sketchy websites, setting up servers, and sharing your config to play with friends. MCVM hopes to alleviate some of these pains with its smart versioning system.

**_Profiles_** in MCVM are groups of settings, modloaders, and the such with a game version. They are shared config for versions of the game which you then attach instances to. **_Instances_** are programs such as a client or server which have a parent profile and are the thing you actually run.

**_Packages_** are the big selling point of MCVM. They are scripts which are configured on a profile and run so that they obtain the correct content files for your game. The `sodium` package, for example, when installed on a profile, will add the Sodium jar file to your mods folder only on client instances.

While MCVM is primarily a command-line tool, it is also available as a Rust library to base other launchers on. You can also implement the packaging standard in order to provide packages for MCVM or install MCVM packages on your launcher. Eventually, there may even be a GUI version of the program.

Right now, the launcher is in an early state. If you see something you want that isn't there, try contributing!

## Things that need to be completed before 1.0.0:

- Authentication / Login
- Installing Forge
- A website and central package repository
