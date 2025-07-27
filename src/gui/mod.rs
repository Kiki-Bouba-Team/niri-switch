mod window_info;

use gio::prelude::*;
use gtk4::prelude::*;
use gtk4_layer_shell::LayerShell;
use window_info::WindowInfo;

/// Creates a gtk selection model for a given vector of niri Windows.
fn create_window_info_model(windows: &Vec<niri_ipc::Window>) -> gtk4::SingleSelection {
    let model = gio::ListStore::new::<WindowInfo>();

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

    /* Upon setup signal, we create empty label for each item in the model */
    factory.connect_setup(move |_, item| {
        let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
        let label = gtk4::Label::new(None);
        item.set_child(Some(&label));
    });

    /* Upon bind signal we set each label text using data stored in the model */
    factory.connect_bind(move |_, item| {
        /* Danger zone: factory bind is a generic function, so quite a bit of casting
         * is neccessery to get to the label widget we created in setup stage.
         * This is barely safe and will panic at runtime if widget structure is modified,
         * so be carefull */
        let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
        let window_info = item.item().and_downcast::<WindowInfo>().unwrap();
        let label = item.child().and_downcast::<gtk4::Label>().unwrap();

        label.set_label(&format!("{}: {}", window_info.id(), window_info.app_id()));
    });

    factory
}

fn activate(application: &gtk4::Application, windows: &Vec<niri_ipc::Window>) {
    let window = gtk4::ApplicationWindow::new(application);

    /* Move this window to the shell layer, this allows to escape Niri compositor
     * and display window on top of everything else */
    window.init_layer_shell();
    window.set_layer(gtk4_layer_shell::Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);

    let selection_model = create_window_info_model(windows);
    let widget_factory = create_window_widget_factory();

    let grid_view = gtk4::GridView::new(Some(selection_model), Some(widget_factory));
    grid_view.set_orientation(gtk4::Orientation::Horizontal);
    grid_view.set_max_columns(1);

    /* GTK Windows are counted references so the following line will not create an actual copy.
     * It will create a second reference that is needed because the following closure takes
     * ownership of the captured variables and would otherwise move out the `window`
     * reference and disallow further access to it from this function */
    let window_ref = window.clone();
    grid_view.connect_activate(move |_, _| {
        /* TODO: focus to selected window */
        window_ref.close();
    });

    window.set_child(Some(&grid_view));

    window.present();
}

pub fn start_gui(windows: Vec<niri_ipc::Window>) {
    let applicaiton_id = "io.kiki_bouba_team.NiriSwitch";
    let application = gtk4::Application::new(Some(applicaiton_id), Default::default());

    application.connect_activate(move |app| activate(&app, &windows));
    application.run();
}
