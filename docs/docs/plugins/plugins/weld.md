# Weld
ID: `weld`

The Weld plugin uses the Weld pack merger to combine resource and data packs in your instances

Note: This plugin requires Python to be installed on your machine

## Usage
Weld is enabled by default for resource and data packs in every instance. It will run every time you update an instance. To disable it on an instance, set the `disable_weld` field on that instance to `true`. To ignore welding certain files, set the `weld_ignore` field on an instance to the names of the files you want to keep intact.
