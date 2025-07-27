# niri-switch

Niri-switch implements faster task switching for Niri compositor. It's meant to resamble Alt-Tab funtionality known from Windows, Gnome, KDE Plasma and many other desktop environments.

## Build and run

Use `cargo build` and `cargo run`. Application is not yet usable in the current state, but it lists windows from current workspace and displays basic GUI.

## Dependencies

* `niri` - obviously, niri-switch needs running Niri instance to connect via IPC socket.
* `gtk4`, `gtk4-layer-shell` - needed to display the graphical interface.

## Todo

- List all windows from workspace (**DONE**)
- Change focus to chosen window (**DONE**)
- GUI for selecting the focused window
- Order windows by the time it was last focused
- Allow for GUI customization

Optionally:
- Daemonize this service and listen for all changes in focus, so that timestamps are always accurate and not based on the changes made by this program alone.
