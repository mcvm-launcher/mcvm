# Config Split
ID: `config_split`

The Config Split plugin allows you to have some parts of your configuration in separate files. It is mostly useful for automation tasks or if you want to keep configuration more spread out. The `instance/template edit` commands have mostly replaced its usage.

## Usage

+++ App
When creating a new instance or template, select Config Split as the plugin for creating it.
+++ CLI
To add new instances and templates to your config, create files called `<instance>.json` or `<template>.json` in your config directory, under the `instances` and `templates` directories. Then, put the JSON configuration for it inside that new file.
+++
