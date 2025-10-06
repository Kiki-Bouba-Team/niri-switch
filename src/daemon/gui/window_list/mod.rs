/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
mod imp;
mod window_info;
mod window_item;

use gtk4::glib;
use gtk4::glib::clone;
use gtk4::subclass::prelude::*;
use gtk4::{SingleSelection, prelude::*};
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

    /// Select the next element in the list, wrap back to the begining if end reached
    pub fn advance_the_selection(&self) {
        let imp = self.imp();
        let selection_model = get_selection_model(&imp.list);
        let list_store = get_list_store(&imp.list);

        let selected = selection_model.selected();
        let new_selected = (selected + 1) % list_store.n_items();
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

    pub fn connect_activate<F: Fn(&gtk4::ListView, u32) + 'static>(&self, f: F) {
        let imp = self.imp();
        imp.list.connect_activate(f);
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

/// Handle the window focus choice
pub fn handle_window_chosen(list: &gtk4::ListView, position: u32, store: &super::GlobalStoreRef) {
    let window_info = list
        .model()
        .expect("List view should have a model")
        .item(position)
        .and_downcast::<WindowInfo>()
        .expect("Model item has to be a 'WindowInfo'");

    /* Create async context and next spawn separate thread that will perform the
     * blocking calls */
    glib::spawn_future_local(clone!(
        #[weak]
        list,
        #[strong]
        store,
        async move {
            let window_id = window_info.id();

            /* Move the chosen window to the front of the window list */
            store.lock().unwrap().window_cache.move_to_front(&window_id);

            /* Socket uses blocking calls, so we create a separete thread */
            gio::spawn_blocking(move || {
                let mut store = store.lock().unwrap();
                store.niri_socket.change_focused_window(window_id);
            })
            .await
            .expect("Blocking call must succeed");

            /* Close the window after changing focus */
            let window = list
                .root()
                .and_downcast::<gtk4::Window>()
                .expect("Root widget has to be a 'Window'");
            window.close()
        }
    ));
}
