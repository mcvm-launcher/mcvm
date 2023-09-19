# MCVM

MCVM is a lightweight Minecraft launcher meant to provide a better interface with custom content such as mods and resource packs.

In the official launcher, you have to waste time moving files in and out of folders, downloading mods from sketchy websites, setting up servers, and sharing your config to play with friends. MCVM hopes to alleviate some of these pains with its smart systems for configuration sharing and package management.

[Documentation](docs/README.md)

## Profiles
**_Profiles_** in MCVM are groups of settings, modloaders, and the such with a game version. They are shared config for versions of the game which you then attach instances to. **_Instances_** are programs such as a client or server which have a parent profile and are the thing you actually run.

## Packages
**_Packages_** are the big selling point of MCVM. They are simple files which are configured on profiles and instances and obtain the correct content files for your game. The `sodium` package, for example, when installed on a profile, will add the Sodium jar file to your mods folder only on client instances.

### A universal format
Packages are designed in such a way that they work with any hosting system. Because they download files from any URL, packages serve as an intermediary for many different websites.

### Package relationships
MCVM's packaging format provides the ability to model complex relationships between packages. You won't have to worry about getting all of the correct dependencies for your packages, as they will be automatically installed.

### Flexibility with scripting
Packages can be more than just an index of files. They can be scripts which run simple logic to determine dependencies and addon files depending on the conditions of the environment.

### Safety
You don't have to worry about your security when using packages. Even though they have scripting capabilities, they are in a controlled environment with no uneeded access to the system. Public repositories will be screened often to ensure quality.

### Control
You don't have to just use the packages from the official repositories. You can use whatever local or remote package repository you please with whatever priority, as long as they match the API standard. The syncing of package files from repositories is a separate process that only happens when you explicitly say so. No worrying about unknown changes from the repositories breaking your game.

## Automatic installation of modifications
Although there is currently only support for a few modifications, such as Fabric, Quilt, and Paper, we hope to eventually install every popular modloader and server implementation automatically.

## Game options management
With the official launcher, changing versions often means your configuration breaks. In most instanced launchers, creating a new instance doesn't bring your options along with it. MCVM combines the best of both.

Global options for your clients and servers can be defined in simple files that propagate seamlessly. Even though Mojang changes the formats for their options files often, MCVM's options are consistent and fully backwards compatible.

## Snapshots and backups
Easily create named backups of the files you want to, and not the ones you don't.

## Support for many types of users
You can log in with XBox Live, as a demo user, or not at all. Support for alternative authentication and skin servers will come in the future.

## Presets and sensible defaults
There are many available presets for popular sets of game options that optimize servers, such as Aikar's or Krusic's. Although you can configure a lot, you don't have to to get a great experience.

## Fast and resource-efficient
MCVM does a lot of work in parallel and is shipped as a single binary without the need for any runtime. The linked instances data model MCVM uses allows separation of data while still sharing large files using hardlinks. Optimizing disk use is a big focus.

## Extremely configurable and modular
MCVM has a deep amount of configuration for pretty much every part of the application. Its availablity as a library, integrations, and flexible command-line interface allow scripting any parts you want. With your permission, packages can access the local filesystem and run commands to fit your needs.

## Compatability and stability as a feature
Where others may take shortcuts, MCVM strives for perfect compatability with Mojang's formats.

## Use cases
MCVM has many use cases for different applications 

### A command-line client launcher
This is the main use case of most people, and is an important focus of the ecosystem.

### A library for your launcher
You can use the MCVM library as a base for the functionalities of your launcher. Even if you don't use the packaging formats, MCVM contains functions to launch the game in a simple way.

### A server management tool
The MCVM CLI is a perfect asset for server managers. The way that it groups configuration for instances makes it easy to orchestrate multiple running servers at once. This system will be great in the future as well when MCVM adds support for proxies like BungeeCord and Velocity.

### A packaging format
Launchers can use the different MCVM crates to parse, validate, evaluate, and host MCVM packages.

### A GUI launcher (planned)
All the functionalities of the CLI in a more approachable format.

## Progress

Right now, the launcher and library have most of the core features implemented. However, support for more complex features and the hosting of the package ecosystem have yet to be fleshed out. If you see something you want that isn't there, try contributing!

## Things that need to be completed before 1.0.0:

- Storing login credentials so you don't have to log in every time you launch
- Installing Forge
- A website and central package repository

Contact `@carbonsmasher` on Discord if you have any questions.
