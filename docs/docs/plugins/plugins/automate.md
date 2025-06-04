# Automate
ID: `automate`

The Automate plugin allows you to attach shell commands to be run when an instance starts or stops.

## Usage
Use the `on_launch` and `on_stop` fields on an instance to specify the commands you want to run. Commands will be run using the default system shell if set, and `/bin/sh` otherwise.
