/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
mod store;
mod style;
mod window_info;
mod window_item;
mod window_list;

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

/* Type aliases to make signatures more readable */
type GlobalStoreRef = Arc<Mutex<store::GlobalStore>>;
type WindowWeakRef = glib::WeakRef<gtk4::ApplicationWindow>;

const GTK4_APP_ID: &str = "org.kikibouba.NiriSwitch";
const CLIENT_REQUEST_CAP: usize = 20;

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

/// Updates the cached window list with new windows, and remove the old ones
fn update_window_cache(windows: &[niri_ipc::Window], store: &GlobalStoreRef) {
    /* Create a set of current window ids */
    let current_id_set: HashSet<u64> = windows.iter().map(|window| window.id).collect();

    let mut store = store.lock().unwrap();
    /* Update the cache with the new id set */
    store.window_cache.update_cache(current_id_set);
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
        window_list::advance_the_selection(list);
        return;
    }
    /* Else reload the listed windows, state might have changed since the last time.
     * This is also the initial filling of the list. */
    window_list::clear_the_list(list);

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
    window_list::fill_the_list(list, &windows, store);

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
    /* Create widget for displaying windows */
    let list_view = window_list::create_window_list(global_store);

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
