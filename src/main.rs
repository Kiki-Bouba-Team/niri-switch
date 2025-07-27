use std::process;

mod connection;
mod gui;

fn main() {
    /* Establish connection with the Niri instance */
    let connection = connection::Connection::new();

    let mut connection = match connection {
        Some(connection) => connection,
        None => {
            eprintln!("Failed to connect with Niri instance");
            process::exit(1);
        }
    };

    /* Get currently focused window - this will be used to determine
     * which workspace we should be operating on */
    let focused_window = connection.get_focused_window();
    let focused_window = match focused_window {
        Some(window) => window,
        None => {
            eprintln!("Unable to retrieve currently focused window");
            process::exit(1)
        }
    };

    let workspace_id = match focused_window.workspace_id {
        Some(id) => id,
        None => {
            eprintln!("Focused window does not have workspace id");
            process::exit(1)
        }
    };

    // /* Get all of the windows from our chosen workspace */
    let windows = connection.list_windows_in_workspace(workspace_id);

    gui::start_gui(windows, connection);

    process::exit(0)
}
