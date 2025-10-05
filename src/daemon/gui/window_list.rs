/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
use super::window_info::WindowInfo;
use super::window_item::WindowItem;

use super::GlobalStoreRef;
use gtk4::glib::clone;
use gtk4::{SingleSelection, prelude::*};
use niri_ipc::Window;

/// Creates GTK list widget for listing windows
pub fn create_window_list(store: &GlobalStoreRef) -> gtk4::ListView {
    let window_store = gio::ListStore::new::<WindowInfo>();
    let selection_model = gtk4::SingleSelection::new(Some(window_store));
    let widget_factory = create_window_widget_factory();

    let list_view = gtk4::ListView::builder()
        .model(&selection_model)
        .factory(&widget_factory)
        .orientation(gtk4::Orientation::Horizontal)
        .single_click_activate(true)
        .css_name("window-list")
        .build();

    /* clone! macro will create another reference to socket object, so it can be moved
     * to the closure. The closure can outlive ther current function scope, so it has hold
     * own reference. */
    list_view.connect_activate(clone!(
        #[strong]
        store,
        move |list, position| handle_window_chosen(list, position, &store)
    ));

    list_view
}

/// Creates a gtk widget factory for displa ying window information.
fn create_window_widget_factory() -> gtk4::SignalListItemFactory {
    /* GTK factory is an object responsible for producing widgets and binding
     * data from the model */
    let factory = gtk4::SignalListItemFactory::new();

    /* Upon setup signal, we create empty widget for each item in the model */
    factory.connect_setup(move |_, item| {
        let window_item = WindowItem::default();

        item.downcast_ref::<gtk4::ListItem>()
            .expect("Needs to be a ListItem")
            .set_child(Some(&window_item));
    });

    /* Upon bind signal we fill widgets using data stored in the model */
    factory.connect_bind(move |_, item| {
        let item = item
            .downcast_ref::<gtk4::ListItem>()
            .expect("Needs to be a ListItem");

        let window_info = item
            .item()
            .and_downcast::<WindowInfo>()
            .expect("The item has to be a 'WindowInfo'");

        let window_item = item
            .child()
            .and_downcast::<WindowItem>()
            .expect("The child needs to be a 'WindowItem'");

        window_item.set_window_info(window_info);
    });

    factory
}

/// Handle the window focus choice
fn handle_window_chosen(list: &gtk4::ListView, position: u32, store: &GlobalStoreRef) {
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

/// Select the next element in the list, wrap back to the begining if end reached
pub fn advance_the_selection(list: &gtk4::ListView) {
    let selection_model = get_selection_model(list);
    let list_store = get_list_store(list);

    let selected = selection_model.selected();
    let new_selected = (selected + 1) % list_store.n_items();
    list.scroll_to(new_selected, gtk4::ListScrollFlags::FOCUS, None);
    list.scroll_to(new_selected, gtk4::ListScrollFlags::SELECT, None);
}

/// Remove all the windows added to the GTK window list
pub fn clear_the_list(list: &gtk4::ListView) {
    let list_store = get_list_store(list);
    list_store.remove_all();
}

/// Given list of niri Windows fill the GTK list of windows
pub fn fill_the_list(list: &gtk4::ListView, windows: &Vec<Window>, store: &super::GlobalStoreRef) {
    let list_store = get_list_store(list);

    for window in windows {
        /* Try to get information about the app that coresponds to the window */
        let window_info = get_widow_info_for_niri_window(window, store);
        list_store.append(&window_info);
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
