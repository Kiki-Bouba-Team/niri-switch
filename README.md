# niri-switch - A niri task switcher

niri-switch implements fast task switching for [niri](https://github.com/YaLTeR/niri) compositor. It aims to provide functionality similar to Alt-Tab known from Windows, Gnome, KDE Plasma and many other desktop environments.

## Project stutus

niri-switch is currently **usable**, but not very **useful**. It is still in early development and requires few features to be completed to actually deliver a good user experience. But anyone is welcome to play around with it and provide much appreciated feedback.

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

**After restarting the niri session, niri-switch is ready to use.**

You can bind the command to any key combination, `Alt+Tab` is just an example.

## Configuration

You can customize the look by providing custom `~/.config/niri-switch/style.css` file. The default configuration is located in `src/daemon/gui/style.css`, you can copy and modify it.

To examine the CSS classes and the widget hierarchy you can run the `niri-switch-daemon` with debug flag: `GTK_DEBUG=interactive niri-switch-daemon` and play around in the inspector.

GTK supports only a specific subset of CSS properties, you can learn more about it in GTK [documentation](https://docs.gtk.org/gtk4/css-properties.html).

The configuration is loaded at the daemon startup in this order:

* `$XDG_CONFIG_HOME/niri-switch/style.css` - if the environment variable is set and file exists.
* `$HOME/.config/niri-switch/style.css` - if the above does not exist or environment variable is not set.
* Embeded `src/daemon/gui/style.css` - if none of the above exist or needed variables are not set.

## Todo

- List all windows from workspaces (**DONE**)
- Change focus to chosen window (**DONE**)
- GUI for selecting the focused window (**DONE**)
- Order windows by the time it was last focused
- Allow for GUI customization (**DONE**)

Optionally:
- Daemonize this service and listen for all changes in focus, so that timestamps are always accurate and not based on the changes made by this program alone.
