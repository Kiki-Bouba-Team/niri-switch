/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */

mod app_database;
mod window_cache;

use crate::niri_socket::NiriSocket;
use app_database::AppDatabase;
use window_cache::WindowCache;

/// Stores objects and information that need to be widely available
/// in the app or is often reused.
pub struct GlobalStore {
    pub niri_socket: NiriSocket,
    pub app_database: AppDatabase,
    pub window_cache: WindowCache,
}

impl GlobalStore {
    pub fn new(niri_socket: NiriSocket) -> Self {
        Self {
            niri_socket,
            app_database: AppDatabase::new(),
            window_cache: WindowCache::new(),
        }
    }
}
