# MCVM

MCVM is a lightweight Minecraft launcher and management ecosystem meant to provide a better interface with custom content such as mods and resource packs.

In the official launcher and many alternative ones, you have to waste time moving files in and out of folders, downloading mods from sketchy websites, setting up servers, and sharing your config to play with friends. MCVM hopes to alleviate some of these pains with its smart systems for configuration sharing and package management.

- [Documentation](docs/README.md)
- [Contributing](CONTRIBUTING.md)

## Instances
You have probably heard of instances before from other launchers like MultiMC. They are separate installations of the game that are kept isolated from each other to prevent your data from combining in strange ways. MCVM supports both client and server instances and makes the management of both trivial.

## Profiles
**_Profiles_** in MCVM are shared configuration for multiple instances which are attached to them. Profiles apply their settings, such as the game version and modloader, to all of the instances that they contain. This lets you update multiple instances simulataneously and sync data between them.

## Packages
**_Packages_** are the big selling point of MCVM. They are simple files which are configured on profiles and instances and obtain the correct content files for your game. The `sodium` package, for example, when installed on a profile, will add the Sodium jar file to your mods folder only on client instances.

### A universal format
Packages are designed in such a way that they work with any hosting system. Because they download files from any URL, packages serve as an intermediary for many different websites.

### Package relationships
MCVM's packaging format provides the ability to model complex relationships between packages. You won't have to worry about getting all of the correct dependencies for your packages, as they will be automatically installed.

### Flexibility with scripting
Packages can be more than just an index of files. They can be scripts which run simple logic to determine dependencies and addon files depending on the conditions of the environment.

### Safety
Packages are made to be as secure as possible. Even though they have scripting capabilities, they are in a controlled environment with no uneeded access to the system or ability to run arbitrary code. Public repositories will be screened often to ensure quality.

### Control
You don't have to just use the packages from the official repositories. You can use whatever local or remote package repository you please with whatever priority, as long as they match the API standard. The syncing of package files from repositories is a separate process that only happens when you explicitly say so. Changes to packages will never break your game without your knowledge.

### Automatic installation of modifications
Although there is currently only support for a few modifications, such as Fabric, Quilt, and Paper, we hope to eventually install every popular modloader, server implementation, and proxy automatically.

### Game options management
With the official launcher, changing versions often means your configuration breaks. In most instanced launchers, creating a new instance doesn't bring your options along with it. MCVM combines the best of both.

Global options for your clients and servers can be defined in simple files that propagate seamlessly. Even though Mojang changes the formats for their options files often, MCVM's options are consistent and fully backwards compatible.

### Snapshots and backups
Easily create named backups of the files you want to, and not the ones you don't.

### Support for many types of users
You can log in with Microsoft, as a demo user, or not at all. You don't need to have an internet connection to play. Support for alternative authentication and skin servers will come in the future.

### Presets and sensible defaults
There are many available presets for popular sets of game options that optimize servers, such as Aikar's or Krusic's. Although you can configure a lot, you don't have to to get a great experience.

### Fast and resource-efficient
MCVM does a lot of work in parallel and is shipped as a single binary without the need for any runtime. The linked instances data model MCVM uses allows separation of data while still sharing large files using hardlinks. Optimizing disk use is a big focus.

### Extremely configurable and modular
MCVM has a deep amount of configuration for pretty much every part of the application. Its availablity as a library, integrations, and flexible command-line interface allow scripting many different parts. With your permission, packages can access the local filesystem and run commands to fit your needs.

### Compatability and stability as a feature
Where others may take shortcuts, MCVM strives for perfect compatability with Mojang's formats.

## Use cases
MCVM has many use cases for different applications 

### A command-line launcher
This is the main use case of most people, and is an important focus of the ecosystem.

### A GUI launcher (planned)
All the functionalities of the CLI in a more approachable format as a desktop application.

### A library for your launcher
You can use the MCVM library as a base for the functionalities of your launcher. Even if you don't use the packaging formats, MCVM contains functions to launch the game in a simple way, as well as customize the launch process to your liking.

### A server management tool
The MCVM CLI is a perfect asset for server managers. The way that it groups configuration for instances makes it easy to orchestrate multiple running servers at once. This system will be great in the future as well when MCVM adds support for proxies like BungeeCord and Velocity.

### A packaging format
Launchers can use the different MCVM crates to parse, validate, evaluate, and host MCVM packages.

### Automated testing and CI
MCVM can be used to quickly start up a server instance in your automated CI pipelines, which you can run tests on.

## Progress

Right now, the launcher and library have most of the core features implemented. However, support for more complex features and the hosting of the package ecosystem have yet to be fleshed out. If you see something you want that isn't there, try contributing!

## Things that need to be completed before 1.0.0:

- Storing login credentials so you don't have to log in every time you launch, along with checking for game ownership so that you can play offline
- Installing Forge

Contact `@carbonsmasher` on Discord if you have any questions.
