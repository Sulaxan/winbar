# DateTime Component

Example JSON:

```
"DateTime": {
    "format": "%F %r",
    "styles": { ... }
}
```

## Fields

| Key      | Description                                                                                     |
| -------- | ----------------------------------------------------------------------------------------------- |
| `format` | The format of the datetime. See https://docs.rs/chrono/latest/chrono/format/strftime/index.html |
| `styles` | The styles to apply to the component. See [styling](../styling.md).                             |
