/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
mod window_info;

use super::CliArgs;
use super::connection::Connection;

use gio::prelude::*;
use gtk4::glib::clone;
use gtk4::prelude::*;
use gtk4_layer_shell::LayerShell;
use niri_ipc::Window;
use std::sync::{Arc, Mutex};
use window_info::WindowInfo;

/* Type aliases to make signatures more readable */
type ConnectionRef = Arc<Mutex<Connection>>;
type WindowWeakRef = glib::WeakRef<gtk4::ApplicationWindow>;

pub const APP_ID: &str = "io.kiki_bouba_team.NiriSwitch";
const WINDOW_LABEL_MARGIN: i32 = 15;

/// Creates a gtk selection model with windows retrieved via niri ipc
fn create_window_info_model(args: &CliArgs, connection: &ConnectionRef) -> gtk4::SingleSelection {
    let model = gio::ListStore::new::<WindowInfo>();
    let mut connection = connection.lock().unwrap();
    /* connection uses blocking calls and this function is run on the
     * main GTK thread so usually that would not be fine, but since
     * this is just the activate stage, it will not freeze anything */
    let mut windows = connection.list_windows();

    /* User can request to only show windows from active workspace */
    if args.workspace {
        let workspace = connection
            .get_active_workspace()
            .expect("Unable to get active workspace");

        let is_window_from_workspace = |window: &Window| -> bool {
            if let Some(id) = window.workspace_id {
                return id == workspace.id;
            };
            return false;
        };

        windows = windows
            .into_iter()
            .filter(is_window_from_workspace)
            .collect();
    }

    for window in windows {
        /* WindowInfo is a glib object that stores information about window */
        model.append(&WindowInfo::new(
            window.id,
            window.app_id.clone().unwrap_or_default().as_str(),
        ));
    }

    gtk4::SingleSelection::new(Some(model))
}

/// Creates a gtk widget factory for displaying window information.
fn create_window_widget_factory() -> gtk4::SignalListItemFactory {
    /* GTK factory is an object responsible for producing widgets and binding
     * data from the model */
    let factory = gtk4::SignalListItemFactory::new();

    /* Upon setup signal, we create box with empty label for each item in the model */
    factory.connect_setup(move |_, item| {
        let label = gtk4::Label::builder()
            .margin_bottom(WINDOW_LABEL_MARGIN)
            .margin_top(WINDOW_LABEL_MARGIN)
            .margin_start(WINDOW_LABEL_MARGIN)
            .margin_end(WINDOW_LABEL_MARGIN)
            .build();
        let box_widget = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .build();
        box_widget.append(&label);

        item.downcast_ref::<gtk4::ListItem>()
            .expect("Needs to be a ListItem")
            .set_child(Some(&box_widget));
    });

    /* Upon bind signal we set each label text using data stored in the model */
    factory.connect_bind(move |_, item| {
        /* Danger zone: factory bind is a generic function, so quite a bit of casting
         * is neccessery to get to the label widget we created in setup stage.
         * This is barely safe and will panic at runtime if widget structure is modified,
         * so be carefull */
        let window_info = item
            .downcast_ref::<gtk4::ListItem>()
            .expect("Needs to be ListItem")
            .item()
            .and_downcast::<WindowInfo>()
            .expect("The item has to be a 'WindowInfo'");

        let label = item
            .downcast_ref::<gtk4::ListItem>()
            .expect("Needs to be ListItem")
            .child()
            .and_downcast::<gtk4::Box>()
            .expect("The child needs to be a 'Box'")
            .first_child()
            .and_downcast::<gtk4::Label>()
            .expect("First child has to be a 'Label'");

        label.set_label(&format!("{}: {}", window_info.id(), window_info.app_id()));
    });

    factory
}

/// Handle the window focus choice
fn window_chosen(list: &gtk4::ListView, position: u32, connection: &ConnectionRef) {
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
        connection,
        async move {
            let window_id = window_info.id();

            /* Connection uses blocking calls, so we create a separete thread */
            gio::spawn_blocking(move || {
                let mut connection = connection.lock().unwrap();
                connection.change_focused_window(window_id);
            })
            .await
            .expect("Blocking call must succeed");

            /* Close the window after changing focus */
            let window = list
                .root()
                .and_downcast::<gtk4::Window>()
                .expect("Root widget has to be a 'Window'");
            window.close();
        }
    ));
}

/// Handle key press events on the main window
fn handle_key_pressed(key: gdk4::Key, window_ref: &WindowWeakRef) -> glib::Propagation {
    let mut propagation = glib::Propagation::Proceed;
    match key {
        gdk4::Key::Escape => {
            let window = window_ref
                .upgrade()
                .expect("Controller shouldn't outlive the window");
            window.close();
        }
        gdk4::Key::Tab => {
            /* Prevent default Tab behaviour */
            propagation = glib::Propagation::Stop;
        }
        _ => (),
    }
    propagation
}

/// Creates the main window, widgets, models and factories
fn activate(application: &gtk4::Application, args: &CliArgs, connection: &ConnectionRef) {
    let selection_model = create_window_info_model(args, connection);
    let widget_factory = create_window_widget_factory();

    let list_view = gtk4::ListView::builder()
        .model(&selection_model)
        .factory(&widget_factory)
        .orientation(gtk4::Orientation::Horizontal)
        .build();

    /* clone! macro will create another reference to connection object, so it can be moved
     * to the closure. The closure can outlive ther current function scope, so it has hold
     * own reference. */
    list_view.connect_activate(clone!(
        #[strong]
        connection,
        move |grid, position| window_chosen(grid, position, &connection)
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

    window.present();
}

/// Start the GUI for choosing next window to focus
pub fn start_gui(args: CliArgs, connection: Connection) {
    /* This use of atomic smart pointer and mutex allow for multiple owners that can
     * acquire the connection object and send requests from the context of different
     * threads. */
    let connection_reference = Arc::new(Mutex::new(connection));

    let application = gtk4::Application::new(Some(APP_ID), Default::default());

    application.connect_activate(move |app| activate(&app, &args, &connection_reference));

    /* Need to pass no arguments explicitely, otherwise gtk will try to parse our
     * custom cli options */
    let no_args: Vec<String> = vec![];
    application.run_with_args(&no_args);
}
