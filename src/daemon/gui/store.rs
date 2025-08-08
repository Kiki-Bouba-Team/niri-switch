/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */

use crate::niri_socket::NiriSocket;
use gio::prelude::{AppInfoExt, IconExt};
use std::collections::{HashSet, VecDeque};

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

/// Window cache keeps track of the window list displayed to the user
/// so that it can be saved and changed if needed.
pub struct WindowCache {
    /// The ID set allows for quick lookups of cached IDs
    window_id_set: HashSet<u64>,
    /// The window ID list keeps track of the order
    window_id_list: VecDeque<u64>,
}

impl WindowCache {
    pub fn new() -> Self {
        Self {
            window_id_set: HashSet::new(),
            window_id_list: VecDeque::new(),
        }
    }

    /// Get set of IDs that are not cached yet
    fn get_new_windows(&self, current_windows: &HashSet<u64>) -> HashSet<u64> {
        current_windows
            .difference(&self.window_id_set)
            .cloned()
            .collect()
    }

    /// Get set of IDs that are cached but are not present in the provided set
    fn get_obsolete_windows(&self, current_windows: &HashSet<u64>) -> HashSet<u64> {
        self.window_id_set
            .difference(&current_windows)
            .cloned()
            .collect()
    }

    /// Given new set of window IDs, update the cached set
    pub fn update_cache(&mut self, current_windows: HashSet<u64>) {
        let obsolete_windows = self.get_obsolete_windows(&current_windows);

        /* Remove all the obsolete windows from the window list */
        for window_id in obsolete_windows {
            let list_index = self
                .window_id_list
                .iter()
                .position(|&x| x == window_id)
                .expect("Window id should be somewhere on the list");
            self.window_id_list
                .remove(list_index)
                .expect("List index should be valid");
        }

        let new_windows = self.get_new_windows(&current_windows);

        /* Add all the new windows */
        for window_id in new_windows {
            self.window_id_list.push_back(window_id);
        }

        /* Overwrite the old window ID set */
        self.window_id_set = current_windows;

        /* Sanity check, just in case, length of both collections should be the same */
        assert_eq!(self.window_id_list.len(), self.window_id_set.len());
    }

    /// Move given window id to the front of the window list
    pub fn move_to_front(&mut self, window_id: &u64) {
        let index = self
            .window_id_list
            .iter()
            .position(|&x| x == *window_id)
            .expect("Invalid window ID should not be passed here");

        self.window_id_list
            .remove(index)
            .expect("Removal od window id should not fail");
        self.window_id_list.push_front(*window_id);
    }
}

impl<'a> IntoIterator for &'a WindowCache {
    type Item = &'a u64;
    type IntoIter = std::collections::vec_deque::Iter<'a, u64>;

    fn into_iter(self) -> Self::IntoIter {
        /* Delagate the iterator of the inner window id list */
        self.window_id_list.iter()
    }
}

/// Stores information about installed aps retrieved from gio
pub struct AppDatabase {
    app_list: Vec<AppInfo>,
}

impl AppDatabase {
    pub fn new() -> Self {
        /* Get information about currently installed applications */
        let app_list = gio::AppInfo::all().iter().map(AppInfo::from).collect();
        Self { app_list }
    }

    pub fn get_app_info(&self, app_id: &String) -> Option<AppInfo> {
        /* Good to try matching the id directly first, for some reason it sometimes
         * works better then the gio method. The gio algorithm is not described
         * anywhere so hard to know why that happens */
        let display_name = self.get_app_info_from_cache(&format!("{}.desktop", app_id));
        if let Some(name) = display_name {
            return Some(name);
        }

        /* Get desktop app names matched to the requested string. The matches come
         * sorted according to the quality of match. Matches with the same quality
         * are put in the same array. The best ones are at the beginning */
        let matches = gio::DesktopAppInfo::search(&app_id);
        let best_matches = matches.get(0)?;

        /* If there are multiple best fit results, choose the shortest one.
         * This is just a heuristic that seems to work fine on average */
        let best_match = best_matches.iter().min_by_key(|app_id| app_id.len())?;

        self.get_app_info_from_cache(&best_match.to_string())
    }

    fn get_app_info_from_cache(&self, requested_id: &String) -> Option<AppInfo> {
        for app_info in &self.app_list {
            if let Some(app_id) = &app_info.app_id {
                if app_id == requested_id {
                    return Some(app_info.clone());
                }
            }
        }
        None
    }
}

/// Type safe wrapper around gio::AppInfo
///
/// This is needed, because gio::AppInfo holds some raw pointers,
/// which are generally not safe to share between threads.
#[derive(Clone)]
pub struct AppInfo {
    pub app_id: Option<String>,
    pub display_name: String,
    /* Serialized gio::Icon */
    pub icon: Option<glib::Variant>,
}

impl From<&gio::AppInfo> for AppInfo {
    fn from(app_info: &gio::AppInfo) -> Self {
        let app_id = match app_info.id() {
            Some(id) => Some(id.to_string()),
            None => None,
        };

        let display_name = app_info.display_name().to_string();

        let icon = match app_info.icon() {
            Some(icon) => icon.serialize(),
            None => None,
        };

        Self {
            app_id,
            display_name,
            icon,
        }
    }
}
