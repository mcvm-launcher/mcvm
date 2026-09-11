# Share
ID: `share`

The Share plugin lets you easily share your configuration or addons with friends to synchronize your experience

## Usage

### Sharing Templates

+++ App
Select the template on the home page and click the three dots at the bottom of the screen. Select export as code and follow the prompts.
+++ CLI
Run `nitro template share <template>` to export a template online and give you a code to copy and share
+++

### Using Shared Templates

+++ App
On the homepage, click `+ New` in the top left and select `Template from Code`. Paste the code in the box and choose an ID for the new template, then click Import.
+++ CLI
Run `nitro template use <code> <id>` to import a shared template and give it <id> as it's new ID in your config
+++

### Sharing Addon Zips

+++ App
Not supported yet.
+++
`nitro instance share-addons <instance> <addon1> <addon2> ...`

- `<instance>` The instance to zip the addons of
- `<addon>` Addon types to include in the zip. Can be one of `mods`, `resource_packs`, `plugins`, or `shaders`.
- By default the output is saved to `./addons.zip`. You can use the `--output` flag to specify another filename if you want.
+++