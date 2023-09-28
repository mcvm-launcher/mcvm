# Contributing to MCVM
Just fork and make a PR, about as simple as that. Try to only work on the `dev` branch, as `main` is for finished releases.

## Project structure
- `/` and `/src`: The root of the project and the `mcvm` crate. This is where all of the core library code is for MCVM. It is split into a handful of large modules that should be pretty self-explanatory.
- `/crates`: Other crates that `mcvm` either uses or is used by.
- `/crates/mcvm_cli`: The command-line interface for MCVM.
- `/crates/mcvm_parse`: Package script parsing.
- `/crates/mcvm_pkg`: Contains all of the standard formats and utilities for dealing with MCVM packages. Has the declarative format, dependency resolution, package script evaluation, the repository format, and meta/props evaluation.
- `/crates/mcvm_shared`: Shared types and utils for all of the MCVM crates that can't really live anywhere else.
- `/crates/mcvm_tools`: A command line utility that uses MCVM to do certain tasks, mostly relating to generating files.
- `/tools`: Some assorted scripts and tools to help development.
