/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
mod imp;
mod window_info;
mod window_item;

use gtk4::glib;
use gtk4::subclass::prelude::*;
use gtk4::{prelude::*, SingleSelection};
use niri_ipc::Window;
use window_info::WindowInfo;

/* Here we create custom widget for displaying window info by
 * subclassing gtk4::Box */
glib::wrapper! {
    pub struct WindowList(ObjectSubclass<imp::WindowList>)
        @extends gtk4::Widget, gtk4::Box,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for WindowList {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl WindowList {
    /// Given list of niri Windows fill the GTK list of windows
    pub fn fill_the_list(&self, windows: &Vec<Window>, store: &super::GlobalStoreRef) {
        let imp = self.imp();
        let list_store = get_list_store(&imp.list);

        for window in windows {
            /* Try to get information about the app that coresponds to the window */
            let window_info = get_widow_info_for_niri_window(window, store);
            list_store.append(&window_info);
        }
    }

    /// Moves the selection in the list by the given direction (positive or negative).
    /// If the new position goes past the end or before the beginning, the selection wraps around.
    pub fn advance_the_selection(&self, direction: i64) {
        let imp = self.imp();
        let selection_model = get_selection_model(&imp.list);
        let list_store = get_list_store(&imp.list);

        let new_selected = i64::from(selection_model.selected()) + direction;
        let new_selected = if new_selected < 0 {
            list_store.n_items() - 1
        } else {
            let number_of_elements = i64::from(list_store.n_items());
            u32::try_from(new_selected % number_of_elements).unwrap()
        };

        imp.list
            .scroll_to(new_selected, gtk4::ListScrollFlags::FOCUS, None);
        imp.list
            .scroll_to(new_selected, gtk4::ListScrollFlags::SELECT, None);
    }

    /// Remove all the windows added to the GTK window list
    pub fn clear_the_list(&self) {
        let imp = self.imp();
        let list_store = get_list_store(&imp.list);
        list_store.remove_all();
    }

    /// Bring focus to the inner list
    pub fn focus_to_list(&self) {
        let imp = self.imp();
        imp.list.grab_focus();
    }
}

/// Retrieves glib selection model from GTK4 window list
fn get_selection_model(list: &gtk4::ListView) -> SingleSelection {
    list.model()
        .expect("ListView needs to have a model")
        .downcast::<gtk4::SingleSelection>()
        .expect("Needs to be a 'SingleSelection' type")
}

/// Retrieves GIO list store from GTK4 window list
fn get_list_store(list: &gtk4::ListView) -> gio::ListStore {
    let selection_model = get_selection_model(list);
    selection_model
        .model()
        .and_downcast::<gio::ListStore>()
        .expect("Needs to be a 'ListStore type")
}

/// Given a niri Window description returns a WindowInfo GObject
fn get_widow_info_for_niri_window(
    window: &niri_ipc::Window,
    store: &super::GlobalStoreRef,
) -> WindowInfo {
    let store = store.lock().unwrap();
    let app_id = window.app_id.clone().unwrap_or_default();

    /* Try to get information about the app that coresponds to the window */
    match store.app_database.get_app_info(&app_id) {
        Some(app_info) => {
            let icon = app_info
                .icon
                .map(|icon| gio::Icon::deserialize(&icon).unwrap());
            WindowInfo::new(window.id, &app_info.display_name, icon)
        }
        None => WindowInfo::new(window.id, &app_id, None),
    }
}
