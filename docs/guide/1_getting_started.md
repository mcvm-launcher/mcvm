# Getting Started With MCVM

This guide will show you how to use the MCVM CLI (Command-line interface) from the basics to more advanced features. This guide will cover most of the important things, but more in-depth documentation can be found on reference pages like [configuring.md](../configuring.md).

## 1. Installing
There are multiple ways to install the CLI. Pick the method that is easiest for you.

### Rust
To install using `cargo`, first install [Rust](https://rustup.rs/). Then run
```sh
cargo install mcvm_cli
```
in your favorite terminal. This will install the CLI on your system.

## 2. Basic Concepts
MCVM has some basic features that need to be explained first.

### Profiles
MCVM allows you to configure profiles, which describe the different properties of game installations. For example, you could configure a Vanilla profile with the latest Minecraft version, or a profile with an older version using a modloader like Fabric.

### Instances
Profiles hold instances, which may be a familiar term that you have heard of before from other launchers. Instances have the same Minecraft version, modloader, and other properties as the profile they are on. The advantage of instances is that they keep worlds and configuration separate between different installations, as opposed to having conflicting files.

## 3. Configuring
Now run the command `mcvm profile list` to create the default config file, and list the example profile and its instances. Now if your run `mcvm config edit`, you should be able to edit the config file in your favorite editor and get a sense of what it looks like. Now, let's try launching one of the default instances.

## 4. Updating
Before you launch, you have to make sure that everything is downloaded and ready first. Run the command `mcvm profile update example` to download all the files that are needed for the instances in the profile. You only have to do this once, or whenever you want to update a profile to new versions of modloaders, packages, etc.

## 5. Launching!
Now we are ready to launch our first instance. When referring to instances in most commands, you have to use an *instance reference*, which is just looks like this: `{profile_id}:{instance_id}`. Run `mcvm instance launch example:client` to start up the client! When launching for the first time, you will have to follow the shown login instructions in order to authenticate with your Microsoft account. Afterwards, you won't have to log in again.

For more info, read the other documentation or join our [Discord server](https://discord.gg/25fhkjeTvW).
