# niri-switch - A niri task switcher

niri-switch implements fast task switching for [niri](https://github.com/YaLTeR/niri) compositor. It aims to provide functionality similar to Alt-Tab known from Windows, Gnome, KDE Plasma and many other desktop environments.

The main use case is quickly switching between windows located on different displays and/or workspaces.

<img width="611" height="96" alt="image" src="https://github.com/user-attachments/assets/c2261156-9ad0-45df-ab25-c6e6964b7dd0" />

## Project status

niri-switch is currently **usable** and quite stable. It is still in early development and requires few features to be completed to actually deliver a good user experience. But anyone is welcome to play around with it and provide much appreciated feedback.

## Dependencies

* `niri` - niri-switch needs a running niri session to connect to it via IPC socket.
* `gtk4`, `gtk4-layer-shell` - needed to display the graphical interface. The minimal required version of GTK4 is `4.12`.

To install the program you will also need `cargo` - the rust build system. It's usually installed via [rustup](https://www.rust-lang.org/tools/install).

## Getting started

Clone the repository and run `cargo install --path ./niri-switch`.

Make sure that `~/.cargo/bin` is in your `$PATH`. You can verify it by running `niri-switch --version`.

Next you need to add following configuration to your niri config at `~/.config/niri/config.kdl`:
```kdl
spawn-at-startup "niri-switch-daemon"

binds {
    // Append to existing binds section
    Alt+Tab { spawn "niri-switch"; }
}
```

**After restarting the niri session, niri-switch will be ready to use.**
> [!NOTE]
> You can bind the command to any key combination, `Alt+Tab` is just an example.

> [!TIP]
> Instead of `spawn-at-startup` you can write a custom systemd service for `graphical-session.target` if you want more control over the daemon.

## Navigation

After opening the overlay (e.g. via Alt + Tab), you can move around with arrow keys and select a window with Enter. To exit without focusing on any window, press Escape.

Reapeted calls to niri-switch will also advance the selection.

## Default themes

niri-switch is based on GTK4 and will use your system's default GTK settings. The config is usually located at `~/.config/gtk-4.0/settings.ini` and can be modified. For example, if you want to use dark theme in niri-switch without any CSS modification, you can add 
```
[Settings]
gtk-application-prefer-dark-theme = true
```
to the `settings.ini` file (this will have a global effect). Or run the `niri-switch-daemon` with `GTK_THEME` environment variable.

## Customization

You can customize the look by providing custom `~/.config/niri-switch/style.css` file. The default configuration is located in `src/daemon/gui/style.css`, you can copy and modify it.

To examine the CSS classes and the widget hierarchy you can run the daemon with debug flag: `GTK_DEBUG=interactive niri-switch-daemon` and play around in the inspector.

GTK supports only a specific subset of CSS properties, you can learn more about it in GTK [documentation](https://docs.gtk.org/gtk4/css-properties.html).

The configuration is loaded at the daemon startup in this order:

* `$XDG_CONFIG_HOME/niri-switch/style.css` - if the environment variable is set and file exists.
* `$HOME/.config/niri-switch/style.css` - if the above does not exist or environment variable is not set.
* Embeded `src/daemon/gui/style.css` - if none of the above exist or needed variables are not set.

## Resources

Very useful materials when working with GTK4 and zbus in Rust:
* [GUI development with Rust and GTK 4](https://gtk-rs.org/gtk4-rs/stable/latest/book/)
* [zbus: D-Bus for Rust made easy](https://dbus2.github.io/zbus/)
