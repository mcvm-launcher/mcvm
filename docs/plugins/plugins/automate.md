# Automate
ID: `automate`

The Automate plugin allows you to attach shell commands to be run when an instance starts or stops.

## Usage

+++ App
In the `Automate Hooks` tab of an instance's config, use the `Before Launch`, `On Launch`, and `After Launch` fields to specify the commands you want to run.
+++ CLI
Use the `before_launch`, `on_launch`, and `on_stop` fields on an instance to specify the commands you want to run. 
+++

Commands will be run using the default system shell if set, and `/bin/sh` on Linux otherwise.
