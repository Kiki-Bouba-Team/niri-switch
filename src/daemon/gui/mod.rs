/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
mod style;
mod window_info;
mod window_item;

use super::dbus;
use super::niri_socket::NiriSocket;

use gio::prelude::*;
use gtk4::glib::clone;
use gtk4::prelude::*;
use gtk4_layer_shell::LayerShell;
use std::sync::{Arc, Mutex};
use window_info::WindowInfo;
use window_item::WindowItem;

/* Type aliases to make signatures more readable */
type NiriSocketRef = Arc<Mutex<NiriSocket>>;
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
fn handle_window_chosen(list: &gtk4::ListView, position: u32, niri_socket: &NiriSocketRef) {
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
        niri_socket,
        async move {
            let window_id = window_info.id();

            /* Socket uses blocking calls, so we create a separete thread */
            gio::spawn_blocking(move || {
                let mut niri_socket = niri_socket.lock().unwrap();
                niri_socket.change_focused_window(window_id);
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
    match key {
        gdk4::Key::Escape => {
            let window = window_ref
                .upgrade()
                .expect("Controller shouldn't outlive the window");
            window.close();
        }
        _ => (),
    }
    glib::Propagation::Proceed
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

/// Handle request to activate the daemon
async fn handle_daemon_activated(list: &gtk4::ListView, niri_socket: &NiriSocketRef) {
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
    let niri_socket_ref = niri_socket.clone();
    let windows = gio::spawn_blocking(move || {
        let mut niri_socket = niri_socket_ref.lock().unwrap();
        niri_socket.list_windows()
    })
    .await
    .expect("Request for windows shouldn't fail");

    /* No need to display anything if there is no window */
    if windows.is_empty() {
        return;
    }

    for window in windows {
        /* WindowInfo is a glib object that stores information about window */
        list_store.append(&WindowInfo::new(
            window.id,
            window.app_id.clone().unwrap_or_default().as_str(),
        ));
    }

    /* Next bring the window back to visibility */
    window.present();

    /* List will loose focus after droping the elements, need to grab it again */
    list.grab_focus();
}

/// Handle event from the D-Bus connection
async fn handle_dbus_event(
    event: dbus::DbusEvent,
    list: &gtk4::ListView,
    niri_socket: &NiriSocketRef,
) {
    use dbus::DbusEvent::*;
    match event {
        Activate => handle_daemon_activated(list, niri_socket).await,
    }
}

/// Creates the main window, widgets, models and factories
fn activate(application: &gtk4::Application, niri_socket: &NiriSocketRef) {
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
        niri_socket,
        move |list, position| handle_window_chosen(list, position, &niri_socket)
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
        niri_socket,
        async move {
            while let Ok(event) = receiver.recv().await {
                handle_dbus_event(event, &list_view, &niri_socket).await;
            }
        }
    ));
}

/// Start the GUI for choosing next window to focus
pub fn start_gui(niri_socket: NiriSocket) {
    /* This use of atomic smart pointer and mutex allow for multiple owners that can
     * acquire the socket object and send requests from the context of different
     * threads. */
    let niri_socket_ref = Arc::new(Mutex::new(niri_socket));

    /* Load GTK resources, this will load the compressed *.ui files */
    gio::resources_register_include!("composite_templates.gresource")
        .expect("Registering resources should not fail");

    let application = gtk4::Application::new(Some(GTK4_APP_ID), Default::default());

    application.connect_startup(|_| style::load_css());
    application.connect_activate(move |app| activate(&app, &niri_socket_ref));

    /* Need to pass no arguments explicitely, otherwise gtk will try to parse our
     * custom cli options */
    let no_args: Vec<String> = vec![];
    application.run_with_args(&no_args);
}
