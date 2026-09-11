# Guardian
ID: `guardian`

The Guardian plugin is an automatic malware scanner for mods and server plugins. It can detect many common patterns of token stealers, IP grabbers, and viruses.

## Usage
By default, Guardian will automatically scan whenever you update an instance, reporting an error if any files do not pass the scan.

To do a manual scan, you can run `nitro guardian scan <file>` to get a full report on the given file.

## Configuring

+++ App
Go to an instance or template's configuration under the `Guardian` section. Here you can enable or disable scanning.
+++ CLI
In your instance / template config:

```
{
    "guardian": {
        "scan": bool
    }
}
```

- `scan`: Whether to enable scanning on this instance. Defaults to `true`.
+++
