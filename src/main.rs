use niri_ipc::{
    Action, Reply, Request, Response, Window,
    socket::{self, Socket},
};
use std::{io, process};

mod gui;

fn unwrap_send_result(send_result: io::Result<Reply>) -> Option<Response> {
    let response = match send_result {
        Ok(response) => response,
        Err(error) => {
            eprintln!("Failed to send request: {error:?}");
            return None;
        }
    };

    let response = match response {
        Ok(response) => response,
        Err(error) => {
            eprintln!("Error response from niri: {error:?}");
            return None;
        }
    };

    Some(response)
}

fn get_focused_window(socket: &mut Socket) -> Option<Window> {
    let request = Request::FocusedWindow;
    let send_result = socket.send(request);

    let response = unwrap_send_result(send_result);

    if let Some(Response::FocusedWindow(window)) = response {
        return window;
    }

    None
}

fn list_workspace_windows(workspace_id: u64, socket: &mut Socket) -> Vec<Window> {
    let request = Request::Windows;
    let send_result = socket.send(request);

    let response = unwrap_send_result(send_result);

    let is_window_from_workspace = |window: &Window| -> bool {
        if let Some(id) = window.workspace_id {
            return id == workspace_id;
        };
        return false;
    };

    if let Some(Response::Windows(windows)) = response {
        return windows
            .into_iter()
            .filter(is_window_from_workspace)
            .collect();
    }

    /* No windows in the workspace. Return empty vector for easier usability
     * of this function */
    Vec::new()
}

#[allow(dead_code)]
fn change_focused_window(new_window_id: u64, socket: &mut Socket) -> bool {
    let request = Request::Action(Action::FocusWindow { id: new_window_id });
    let send_result = socket.send(request);

    let response = unwrap_send_result(send_result);

    if let Some(Response::Handled) = response {
        return true;
    };

    false
}

fn main() {
    // Connect to the default niri socket
    let connect_result = socket::Socket::connect();

    let mut connected_socket = match connect_result {
        Ok(socket) => socket,
        Err(error) => {
            eprintln!("Failed to connect with niri socket: {error:?}");
            process::exit(1)
        }
    };

    /* Get currently focused window - this will be used to determine
     * which workspace we should be operating on */
    let focused_window = get_focused_window(&mut connected_socket);
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

    /* Get all of the windows from our chosen workspace */
    let windows = list_workspace_windows(workspace_id, &mut connected_socket);

    gui::start_gui(windows);

    process::exit(0)
}
