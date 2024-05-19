# Styling

Components, in general, take in a styling object to change the look and feel of the component.
Styling in `winbar` is similar to that of CSS in web development, however, the options are much
narrower.

Below is the spec of all fields a styling object can take. You're able to include all or none of
these fields - choose whatever makes sense for your config.

```
"styles": {
    "bg_color": null | Color,
    "fg_color": null | Color,
    "border_style": "Square" | "Round": { "radius": int },
    "font": null | String,
    "font_size": null | int,
    "padding_x": null | int
}
```

Note: instead of specifying `null`, you could choose to omit the field entirely.

## Colors

Color objects can be specified in one of two ways: `inline` or `object`.

### A note on transparency

To indicate something as transparent, you would use `transparent()` in place of the color.
Transparency is handled slightly differently internally, and thus this special type exists.

If you're wondering why the parantheses exists, it's because it uses the `inline` style format.

### Inline

_This is the recommended way to define colors in your config._ Inline styles are specified as a
single string, as opposed to an object. They are are specified in the following format:
`color_function(color)` (akin to how you'd define colors in CSS). `color` is specified in the format
expected by the `color_function`. This may seem a bit complicated, but is quite simple in practice!
Below is a table of all color functions, the format of the color they take, and some examples.

| Function      | Value                                                          | Examples                                  |
| ------------- | -------------------------------------------------------------- | ----------------------------------------- |
| `hex`         | Color in hex using # notation                                  | `hex(#ffffff)`, `hex(#ffffff00)`          |
| `rgb`         | Color using `r, g, b` notation (or, optionally, spaces)        | `rgb(0, 0, 0)`, `rgb(255 255 255)`        |
| `rgba`        | Color using `r, g, b, alpha` notation (or, optionally, spaces) | `rgb(0, 0, 0, 0)`, `rgb(255 255 255 255)` |
| `transparent` | N/A                                                            | `transparent()`                           |

### Object

Though not the preferred method of defining colors, you could choose to define colors using object
notation as well.

#### Hex

```
{
    "Hex": "#FFFFFF" | "#FFFFFF00",
}
```

Note the optional alpha (last 2 hex digits).

#### RGB

```
{
    "Rgb": {
        "r": int,
        "g": int,
        "b": int
    }
}
```

#### RGBA

```
{
    "Rgba": {
        "r": int,
        "g": int,
        "b": int,
        "alpha": int
    }
}
```
