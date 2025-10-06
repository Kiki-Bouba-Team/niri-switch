/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
use super::window_info::WindowInfo;
use super::window_item::WindowItem;
use glib::subclass::InitializingObject;
use gtk4::subclass::prelude::*;

use gtk4::prelude::*;

/* Custom widget for displaying app list created by wrapping ListView.
 * The widget layout will be loaded from the app_list.ui file. Wrapping
 * is needed because otherwise we would not be able to define custom signals. */
#[derive(Debug, Default, gtk4::CompositeTemplate)]
#[template(resource = "/org/kikibouba/niriswitch/window_list/window_list.ui")]
pub struct WindowList {
    #[template_child]
    pub list: TemplateChild<gtk4::ListView>,
}

#[glib::object_subclass]
impl ObjectSubclass for WindowList {
    const NAME: &'static str = "WindowList";
    type Type = super::WindowList;
    type ParentType = gtk4::Box;

    fn class_init(class: &mut Self::Class) {
        class.bind_template();
        class.set_css_name("window-list-wrapper");
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for WindowList {
    fn constructed(&self) {
        self.parent_constructed();
        let window_store = gio::ListStore::new::<WindowInfo>();
        let selection_model = gtk4::SingleSelection::new(Some(window_store));
        let widget_factory = create_window_widget_factory();

        self.list.set_factory(Some(&widget_factory));
        self.list.set_model(Some(&selection_model));
    }
}

impl WidgetImpl for WindowList {}
impl BoxImpl for WindowList {}

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
