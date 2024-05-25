# Plugin Format

## Files
The plugin file format is pretty simple. Inside the plugins directory (`MCVM_DATA/plugins`), all you need is a **manifest** file, located either at `plugins/plugin_id.json` or `plugins/plugin_id/plugin.json`. The nested location allows you to bundle other assets along with your plugin easily, but both locations work exactly the same.

If your plugin needs to run an executable, you can bundle your executable in your plugin directory, or install it on the system.

## Hooks
Hooks are the meat and potatoes of plugins. They allow you to inject into specific points of MCVM's functionality, adding new features. They can act like event handlers, or like data-driven extensions to MCVM's data.
