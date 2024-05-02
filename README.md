# winbar

A Windows 10/11 Status Bar.

`winbar` is quite different from other status bars in that it provides more straightforward
configuration. What this means is that you spend less time configuring the status bar, and have
something that just _works_ and generally looks good.

Of course, you can still change colours and formats of components (available configuration depends
on the component), but you wouldn't be able to change every little detail about how the component is
displayed.

## How does it work?

`winbar` is divided into two components: the CLI (`winbarc`) and the actual status bar app itself
(`winbar`). The CLI can be used to control `winbar`.

`winbar` (the app) runs a TCP server for clients to connect to (in this case, just `winbarc`). It
uses a simple JSON-based request-response protocol. If you're curious/want to make your own client,
see [protocol.rs](./winbar/src/protocol.rs).

## Components

Components are an individual block within the status bar displaying some specific thing. For
example, the datetime component, as the name suggests, displays the current date and time within the
status bar.

### Available Components

The following components are available natively by `winbar`:

- Static Text - displays some static text
- DateTime - displays the current date/time

## Configuration

todo

## Roadmap

- Multi-monitor support
- Plugin system

## Credits

This project's layout and bits of code are inspired from the following:

- [komorebi](https://github.com/LGUG2Z/komorebi)

  - This project's high-level architecture is very similar to that of `komorebi` (separating
    CLI/actual status bar)
  - The window creation/drawing code structure in `winbar` was inspired by `komorebi`

- [gdiplus-rs](https://github.com/davidrios/gdiplus-rs)

  - `winbar`'s GDI+ setup and usage was greatly inspired by `gdiplus-rs`
