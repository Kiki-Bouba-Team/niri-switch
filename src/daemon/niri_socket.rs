/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
use std::io;

/* Use niri_ipc crate provided by niri maintainer <3 */
use niri_ipc::{Action, Reply, Request, Response, Window, Workspace, socket::Socket};

pub struct NiriSocket {
    socket: Socket,
}

impl NiriSocket {
    pub fn new() -> Option<Self> {
        // Connect to the default niri socket
        let connect_result = Socket::connect();

        let connected_socket = match connect_result {
            Ok(socket) => socket,
            Err(error) => {
                eprintln!("Failed to connect with niri socket: {error:?}");
                return None;
            }
        };

        Some(NiriSocket {
            socket: connected_socket,
        })
    }

    #[allow(dead_code)]
    pub fn get_active_workspace(&mut self) -> Option<Workspace> {
        let request = Request::Workspaces;
        let send_result = self.socket.send(request);

        let response = unwrap_send_result(send_result);

        if let Some(Response::Workspaces(workspaces)) = response {
            for workspace in workspaces {
                if !workspace.is_active {
                    continue;
                }
                return Some(workspace);
            }
        };
        None
    }

    pub fn list_windows(&mut self) -> Vec<Window> {
        let request = Request::Windows;
        let send_result = self.socket.send(request);

        let response = unwrap_send_result(send_result);

        if let Some(Response::Windows(windows)) = response {
            return windows;
        }

        /* No windows in the workspace. Return empty vector for easier usability
         * of this function */
        Vec::new()
    }

    pub fn change_focused_window(&mut self, new_window_id: u64) -> bool {
        let request = Request::Action(Action::FocusWindow { id: new_window_id });
        let send_result = self.socket.send(request);

        let response = unwrap_send_result(send_result);

        if let Some(Response::Handled) = response {
            return true;
        };

        false
    }
}

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
