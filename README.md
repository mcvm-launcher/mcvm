# 🚀 MCVM

MCVM is a lightweight Minecraft launcher and management ecosystem meant to provide a better interface with custom content such as mods and resource packs.

In the official launcher and many alternative ones, you have to waste time moving files in and out of folders, downloading mods from sketchy websites, setting up servers, and sharing your config to play with friends. MCVM hopes to alleviate some of these pains with its smart systems for configuration sharing and package management.

In addition, MCVM has a ton of flexibility and power, allowing you to build your perfect launcher by hand, or just use the amazing tools already built by the community.

- 📖 [Documentation](docs/README.md)
- ✨ [Features](#✨-features)
- 📥 [Installation](#📥-installation)
- ➕ [More Info](#➕-more-info)
- 👷 [Use Cases](#👷-use-cases)
- 👍 [Status](#👍-status)
- 🤝 [Contributing](CONTRIBUTING.md)

# ✨ Features

- 🚀 **Launching**: Configure and launch both clients and servers seamlessly.
- 🔌**Plugin System**: Many of MCVM's features are split into separate plugins with a simple and extremely extensible format. 
- ⌨️ **CLI**: An intuitive and ergonomic command-line interface makes using MCVM easy and satisfying.
- 💼 **Instances**: Separate game installations into self-contained instances.
- 🗃️ **Profiles**: Easily share configuration across multiple instances.
- 📦 **Packages**: Automatically install mods, resource packs, and other addons with a novel package format and intelligent dependency management.
- 📥 **Install Everything**: Utilize many of the popular loaders, like Fabric and Quilt, along with server implementations like Paper, with automatic installation.
- 🪪 **User Management**: Configure as many different types of users as you want, and log them in and out as needed.
- 📄 **Game Options**: Specify client options and server properties using a backwards compatible format that can be shared between all your instances.
- 💾 **Backups**: Create named and archived snapshots of the files you want, and not the ones you don't.
- ⚡**Speed**: Probably one of the fastest launchers on the market. Download files concurrently using the best APIs available.
- 🛠️ **Deep Configuration**: Sensible defaults, but plenty of options and escape hatches to make MCVM work for you.
- 🔒 **Robustness**: A lot of design work has gone into making MCVM error-resilient, secure, and future-proof.
- ✅ **Compatability**: MCVM is designed to work on as many operating systems and architectures as possible.

# 📥 Installation
To install the command-line interface, first install [Rust](https://rustup.rs/). Then run
```sh
cargo install mcvm_cli
```
in your favorite terminal. 

# ➕ More Info

### 📦 Packages
**_Packages_** are the big selling point of MCVM. They are simple files which are configured on profiles and instances and obtain the correct content files for your game. The `sodium` package, for example, when installed on a profile, will add the Sodium jar file to your mods folder only on client instances.

#### 🌐 A universal format
Packages are designed in such a way that they work with any hosting system. Because they can download files from any URL, packages serve as an intermediary for the formats and conventions of many different websites.

#### 🔗 Package relationships
MCVM's packaging format provides the ability to model complex relationships between packages. You won't have to worry about getting all of the correct dependencies for your packages, as they will be automatically installed.

#### 🧮 Flexibility with scripting
Packages can be more than just an index of files. They can be scripts which run simple logic to determine dependencies and addon files depending on the conditions of the environment.

#### 🔒 Safety
Packages are made to be as secure as possible. Even though they have scripting capabilities, they are in a controlled environment with no uneeded access to the system or ability to run arbitrary code. Public repositories will be screened often to ensure quality.

#### 🔧 Control
You don't have to just use the packages from the official repositories. You can use whatever local or remote package repository you please with whatever priority, as long as they match the API standard. The syncing of package files from repositories is a separate process that only happens when you explicitly say so. Changes to packages will never break your game without your knowledge.

### 📄 Game options management
With the official launcher, changing versions often means your configuration breaks. In most instanced launchers, creating a new instance doesn't bring your options along with it. MCVM combines the best of both.

Global options for your clients and servers can be defined in simple files that propagate seamlessly. Even though Mojang changes the formats for their options files often, MCVM's options are consistent and fully backwards compatible.

# 👷 Use Cases
MCVM has many use cases for different applications 

### ⌨️ A command-line launcher
This is the main use case of most people, and is an important focus of the ecosystem.

### ✨ A GUI launcher (planned)
All the functionalities of the CLI in a more approachable format as a desktop application.

### 📗 A library for your launcher
You can use the MCVM library as a base for the functionalities of your launcher. Even if you don't use the packaging formats, MCVM contains functions to launch the game in a simple way, as well as customize the launch process to your liking.

### 🪛 A server management tool
The MCVM CLI is a perfect asset for server managers. The way that it groups configuration for instances makes it easy to orchestrate multiple running servers at once. This system will be great in the future as well when MCVM adds support for proxies like BungeeCord and Velocity.

### 📦 A packaging format
Launchers can use the different MCVM crates to parse, validate, evaluate, and host MCVM packages.

# 👍 Status

Right now, the launcher and library have most of the core features implemented. However, support for more complex features such as a plugin system have yet to be fleshed out. If you see something you want that isn't there, try contributing!

### Things that need to be completed before 1.0.0:

- Installing NeoForge
- More in-depth plugin system and a standard plugin set

Join the [Discord](https://discord.gg/25fhkjeTvW) if you have any questions.
