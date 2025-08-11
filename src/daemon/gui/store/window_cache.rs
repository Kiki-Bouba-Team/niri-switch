/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
use std::collections::{HashSet, VecDeque};

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
            .difference(current_windows)
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

impl Default for WindowCache {
    fn default() -> Self {
        Self::new()
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
