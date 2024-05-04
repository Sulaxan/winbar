# Configuring winbar

`winbar` uses a JSON file for configuration. The [winbar.json](../winbar.json) file located in the
root of this repo contains an example config you can use to get started with winbar. You can
optionally generate the same file using:

```
winbar --config-path <PATH> --generate-config
```

Note the usage of `winbar` as opposed to `winbarc`.

## Configuring Components

All components start with some general metadata:

```
{
    "location": "LEFT" | "MIDDLE" | "RIGHT",
    "component": ...
}
```

For the `component` key, see the specific component documentation, which can be found in one of the files
below:

- [Static Text](./components/static_text.md)
- [DateTime](./components/datetime.md)
