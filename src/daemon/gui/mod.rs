/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
mod store;
mod style;
mod window_info;
mod window_item;

use super::dbus;
use super::niri_socket::NiriSocket;

use gio::prelude::*;
use gtk4::glib::clone;
use gtk4::prelude::*;
use gtk4_layer_shell::LayerShell;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};
use window_info::WindowInfo;
use window_item::WindowItem;

/* Type aliases to make signatures more readable */
type GlobalStoreRef = Arc<Mutex<store::GlobalStore>>;
type WindowWeakRef = glib::WeakRef<gtk4::ApplicationWindow>;

const GTK4_APP_ID: &str = "org.kikibouba.NiriSwitch";
const CLIENT_REQUEST_CAP: usize = 20;

/// Creates a gtk widget factory for displaying window information.
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

/// Handle key press events on the main window
fn handle_key_pressed(key: gdk4::Key, window_ref: &WindowWeakRef) -> glib::Propagation {
    if key == gdk4::Key::Escape {
        let window = window_ref
            .upgrade()
            .expect("Controller shouldn't outlive the window");
        window.close();
    }
    glib::Propagation::Proceed
}

/// Given a niri Window description returns a WindowInfo GObject
fn get_widow_info_for_niri_window(window: &niri_ipc::Window, store: &GlobalStoreRef) -> WindowInfo {
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

/// Updates the cached window list with new windows, and remove the old ones
fn update_window_cache(windows: &[niri_ipc::Window], store: &GlobalStoreRef) {
    /* Create a set of current window ids */
    let current_id_set: HashSet<u64> = windows.iter().map(|window| window.id).collect();

    let mut store = store.lock().unwrap();
    /* Update the cache with the new id set */
    store.window_cache.update_cache(current_id_set);
}

/// Select the next element in the list, wrap back to the begining if end reached
fn advance_the_selection(list: &gtk4::ListView) {
    let selection_model = list
        .model()
        .expect("ListView needs to have a model")
        .downcast::<gtk4::SingleSelection>()
        .expect("Needs to be a 'SingleSelection' type");
    let list_store = selection_model
        .model()
        .and_downcast::<gio::ListStore>()
        .expect("Needs to be a 'ListStore type");

    let selected = selection_model.selected();
    let new_selected = (selected + 1) % list_store.n_items();
    list.scroll_to(new_selected, gtk4::ListScrollFlags::FOCUS, None);
    list.scroll_to(new_selected, gtk4::ListScrollFlags::SELECT, None);
}

/// Put the windows in the cached positions
fn sort_windows_by_cached_order(windows: &mut [niri_ipc::Window], store: &GlobalStoreRef) {
    let store = store.lock().unwrap();

    /* Create a lookup table that connects window id to the position in cached list */
    let index_lookup: HashMap<u64, usize> = store
        .window_cache
        .into_iter()
        .enumerate()
        .map(|(idx, id)| (*id, idx))
        .collect();

    /* Sort the windows by the indices */
    windows.sort_by_key(|window| index_lookup.get(&window.id).unwrap());
}

/// Handle request to activate the daemon
async fn handle_daemon_activated(list: &gtk4::ListView, store: &GlobalStoreRef) {
    let window = list
        .root()
        .and_downcast::<gtk4::Window>()
        .expect("Root widget has to be a 'Window'");

    /* If window is already shown, simply advance the selection */
    if window.is_visible() {
        advance_the_selection(list);
        return;
    }
    /* Else relad the window list and present the window */

    let selection_model = list
        .model()
        .expect("ListView needs to have a model")
        .downcast::<gtk4::SingleSelection>()
        .expect("Needs to be a 'SingleSelection' type");
    let list_store = selection_model
        .model()
        .and_downcast::<gio::ListStore>()
        .expect("Needs to be a 'ListStore type");

    /* Reload the listed windows, state might have changes since the last time.
     * This is also the initial filling of the list. */
    list_store.remove_all();

    /* niri socket uses blocking calls, so it will be run on a separate thread */
    let store_ref = store.clone();
    let mut windows = gio::spawn_blocking(move || {
        let mut store = store_ref.lock().unwrap();
        store.niri_socket.list_windows()
    })
    .await
    .expect("Request for windows shouldn't fail");

    /* No need to display anything if there is no window */
    if windows.is_empty() {
        return;
    }

    /* Window list could have changed since the last time */
    update_window_cache(&windows, store);

    /* Put windows in positions that they were last time */
    sort_windows_by_cached_order(&mut windows, store);

    /* If there is more then one window, swap the first two */
    if windows.len() > 1 {
        windows.swap(0, 1);
    }

    /* Append windows to the list model */
    for window in &windows {
        /* Try to get information about the app that coresponds to the window */
        let window_info = get_widow_info_for_niri_window(window, store);
        list_store.append(&window_info);
    }

    /* Next bring the window back to visibility */
    window.present();

    /* List will loose focus after droping the elements, need to grab it again */
    list.grab_focus();
}

/// Handle event from the D-Bus connection
async fn handle_dbus_event(event: dbus::DbusEvent, list: &gtk4::ListView, store: &GlobalStoreRef) {
    use dbus::DbusEvent::*;
    match event {
        Activate => handle_daemon_activated(list, store).await,
    }
}

/// Creates the main window, widgets, models and factories
fn activate(application: &gtk4::Application, global_store: &GlobalStoreRef) {
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
        global_store,
        move |list, position| handle_window_chosen(list, position, &global_store)
    ));

    /* Create main window */
    let window = gtk4::ApplicationWindow::builder()
        .application(application)
        .child(&list_view)
        .build();

    /* Create a weak reference to the window, this will be moved to keyboard controller
     * which will later be attached to the window - with strong referance this could
     * potentially cause a reference cycle and memory leak */
    let window_ref = window.downgrade();
    let keyboard_controller = gtk4::EventControllerKey::new();
    keyboard_controller
        .connect_key_pressed(move |_, key, _, _| handle_key_pressed(key, &window_ref));

    window.add_controller(keyboard_controller);

    /* Move this window to the shell layer, this allows to escape Niri compositor
     * and display window on top of everything else */
    window.init_layer_shell();
    window.set_layer(gtk4_layer_shell::Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
    window.set_hide_on_close(true);

    /* DBus server will communicate with GTK app via async channel */
    let (sender, receiver) = async_channel::bounded(CLIENT_REQUEST_CAP);

    /* Start dbus server for communication with client app */
    glib::spawn_future_local(async move {
        dbus::server_loop(sender)
            .await
            .expect("DBus server shouldn't fail");
    });

    /* Start a task that handles events from D-Bus */
    glib::spawn_future_local(clone!(
        #[weak]
        list_view,
        #[strong]
        global_store,
        async move {
            while let Ok(event) = receiver.recv().await {
                handle_dbus_event(event, &list_view, &global_store).await;
            }
        }
    ));
}

/// Start the GUI for choosing next window to focus
pub fn start_gui(niri_socket: NiriSocket) {
    /* This use of atomic smart pointer and mutex allow for multiple owners that can
     * acquire the store object and mutate it from the context of different threads */
    let store_ref = Arc::new(Mutex::new(store::GlobalStore::new(niri_socket)));

    /* Load GTK resources, this will load the compressed *.ui files */
    gio::resources_register_include!("composite_templates.gresource")
        .expect("Registering resources should not fail");

    let application = gtk4::Application::new(Some(GTK4_APP_ID), Default::default());

    application.connect_startup(|_| style::load_css());
    application.connect_activate(move |app| activate(app, &store_ref));

    /* Need to pass no arguments explicitely, otherwise gtk will try to parse our
     * custom cli options */
    let no_args: Vec<String> = vec![];
    application.run_with_args(&no_args);
}
