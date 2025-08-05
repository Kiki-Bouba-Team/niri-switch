/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */

use crate::niri_socket::NiriSocket;

/// Stores objects that need to be widely available in the app
pub struct GlobalStore {
    pub niri_socket: NiriSocket,
}

impl GlobalStore {
    pub fn new(niri_socket: NiriSocket) -> Self {
        Self { niri_socket }
    }
}
