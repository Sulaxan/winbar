# DateTime Component

Example JSON:

```
"DateTime": {
    "format": "%F %r",
    "bg_color": {
        "Rgb": {
            "r": 23,
            "g": 23,
            "b": 23
        }
    },
    "fg_color": {
        "Rgb": {
            "r": 33,
            "g": 181,
            "b": 80
        }
    }
}
```

## Fields

| Key        | Description                                                                                     |
| ---------- | ----------------------------------------------------------------------------------------------- |
| `format`   | The format of the datetime. See https://docs.rs/chrono/latest/chrono/format/strftime/index.html |
| `bg_color` | The background color of the component                                                           |
| `fg_color` | The foreground color of the component                                                           |
