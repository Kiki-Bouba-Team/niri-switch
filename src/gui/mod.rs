/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
mod window_info;

use super::CliArgs;
use super::connection::Connection;

use gio::prelude::*;
use gtk4::prelude::*;
use gtk4_layer_shell::LayerShell;
use niri_ipc::Window;
use std::{cell::RefCell, rc::Rc};
use window_info::WindowInfo;

type ConnectionRef = Rc<RefCell<Connection>>;

pub const APP_ID: &str = "io.kiki_bouba_team.NiriSwitch";

/// Creates a gtk selection model with windows retrieved via niri ipc
fn create_window_info_model(args: &CliArgs, connection: &ConnectionRef) -> gtk4::SingleSelection {
    let model = gio::ListStore::new::<WindowInfo>();
    let mut connection = connection.borrow_mut();
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
            window.app_id.clone().unwrap().as_str(),
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
        let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
        let box_widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let label = gtk4::Label::new(None);
        box_widget.append(&label);
        item.set_child(Some(&box_widget));
    });

    /* Upon bind signal we set each label text using data stored in the model */
    factory.connect_bind(move |_, item| {
        /* Danger zone: factory bind is a generic function, so quite a bit of casting
         * is neccessery to get to the label widget we created in setup stage.
         * This is barely safe and will panic at runtime if widget structure is modified,
         * so be carefull */
        let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
        let window_info = item.item().and_downcast::<WindowInfo>().unwrap();
        let box_widget = item.child().and_downcast::<gtk4::Box>().unwrap();
        let label = box_widget
            .first_child()
            .and_downcast::<gtk4::Label>()
            .unwrap();

        label.set_label(&format!("{}: {}", window_info.id(), window_info.app_id()));
    });

    factory
}

fn activate(application: &gtk4::Application, args: &CliArgs, connection: &ConnectionRef) {
    let window = gtk4::ApplicationWindow::new(application);

    /* Move this window to the shell layer, this allows to escape Niri compositor
     * and display window on top of everything else */
    window.init_layer_shell();
    window.set_layer(gtk4_layer_shell::Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);

    let selection_model = create_window_info_model(args, connection);
    let widget_factory = create_window_widget_factory();

    let grid_view = gtk4::GridView::new(Some(selection_model), Some(widget_factory));
    grid_view.set_orientation(gtk4::Orientation::Horizontal);
    grid_view.set_max_columns(1);

    /* We need to create another reference to connection object, so it can be moved to
     * the following closure. The closure can outlive current scope, so it has have
     * own reference. */
    let connection_ref = connection.clone();
    grid_view.connect_activate(move |grid, position| {
        let model = grid.model().unwrap();
        let window_info = model.item(position).and_downcast::<WindowInfo>().unwrap();

        let mut connection = connection_ref.borrow_mut();
        connection.change_focused_window(window_info.id());

        let window = grid.root().and_downcast::<gtk4::Window>().unwrap();
        window.close();
    });

    window.set_child(Some(&grid_view));

    window.present();
}

pub fn start_gui(args: CliArgs, connection: Connection) {
    /* This use of smart pointers allow for multiple owners that can borrow
     * connection object and send requests. */
    let connection_reference = Rc::new(RefCell::new(connection));

    let application = gtk4::Application::new(Some(APP_ID), Default::default());

    application.connect_activate(move |app| activate(&app, &args, &connection_reference));

    /* Need to pass no arguments explicitely, otherwise gtk will try to parse our
     * custom cli options */
    let no_args: Vec<String> = vec![];
    application.run_with_args(&no_args);
}
