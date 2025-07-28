# niri-switch - A niri task switcher

Niri-switch implements faster task switching for Niri compositor. It's meant to resamble Alt-Tab funtionality known from Windows, Gnome, KDE Plasma and many other desktop environments.

## Dependencies

* `niri` - obviously, niri-switch needs running Niri instance to connect via IPC socket.
* `gtk4`, `gtk4-layer-shell` - needed to display the graphical interface.

## Build and install

Build the application with `cargo run --release`.
The result binary will be located in `target/release/niri-switch`.

You can add it to your system's `$PATH` any way prefered. If not, then you need to provide full path to the binary in the next step.

Next you can add the following line to niri's binds section in `config.kdl`:
```
binds {
    Alt+Tab { spawn "niri-switch"; }
}
```

And the app is ready to run.

## Options

You can optionally add `--workspace` option to `niri-switch` to limit listed windows to active workspace.

## Todo

- List all windows from workspace (**DONE**)
- Change focus to chosen window (**DONE**)
- GUI for selecting the focused window (**DONE**)
- Order windows by the time it was last focused
- Allow for GUI customization

Optionally:
- Daemonize this service and listen for all changes in focus, so that timestamps are always accurate and not based on the changes made by this program alone.
