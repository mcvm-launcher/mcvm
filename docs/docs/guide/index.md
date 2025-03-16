# Getting Started with MCVM

This guide will show you how to use the MCVM CLI (Command-line interface) from the basics to more advanced features. This guide will cover most of the important things, but more in-depth documentation can be found on reference pages like [configuring.md](../configuring.md).

## 1. Installing
There are multiple ways to install the CLI. Pick the method that is easiest for you.

### Rust
To install using `cargo`, first install [Rust](https://rustup.rs/). Then run
```sh
cargo install mcvm_cli
```
in your favorite terminal. This will install the CLI on your system.

### Releases
Download the correct binary for your system from [the latest release](https://github.com/mcvm-launcher/mcvm/releases/latest).
Note that you will have to install it yourself.

### Dev Builds
To install from one of the prebuilt development binaries, visit [nightly.link](https://nightly.link/mcvm-launcher/mcvm/workflows/build/dev) and download and extract the artifacts for your operating system. Note that these builds may be unstable.

## 2. Basic Concepts
MCVM has some basic features that need to be explained first.

### Instances
Instances may be a familiar term that you have heard of before from other launchers. They are separate game installations with their own Minecraft version, modloader, and other properties. They are also the thing you actually launch when you want to play the game. The advantage of instances is that they keep worlds and configuration separate between different installations, as opposed to having conflicting files.

## 3. Configuring
Run the command `mcvm instance list` to create the default config file, and list the example instances. Now if you run `mcvm config edit`, you should be able to edit the config file in your favorite editor and get a sense of what it looks like. Finally, let's try launching one of the default instances.

## 4. Launching!
Looks like we are ready to launch. Run `mcvm instance launch example-client` to start up the client! When launching for the first time, you will have to follow the shown login instructions in order to authenticate with your Microsoft account. Afterwards, you won't have to log in again.

For more info, read the other documentation or join our [Discord server](https://discord.gg/25fhkjeTvW).
