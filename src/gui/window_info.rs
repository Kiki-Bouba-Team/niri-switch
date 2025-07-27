use glib::Properties;
use glib::prelude::*;
use glib::subclass::prelude::*;
use std::cell::RefCell;

/* This file defines glib object for holding information
 * about detected window. It's entirely based on an exemplary
 * implementation of object subclass found in gtk-rs-core repository. */

mod imp {
    use super::*;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::WindowInfo)]
    pub struct WindowInfo {
        #[property(get, set)]
        id: RefCell<u64>,

        #[property(get, set)]
        app_id: RefCell<String>,
    }

    #[glib::derived_properties]
    impl ObjectImpl for WindowInfo {}

    #[glib::object_subclass]
    impl ObjectSubclass for WindowInfo {
        const NAME: &'static str = "WindowInfo";
        type Type = super::WindowInfo;
    }
}

glib::wrapper! {
    pub struct WindowInfo(ObjectSubclass<imp::WindowInfo>);
}
impl WindowInfo {
    pub fn new(id: u64, app_id: &str) -> Self {
        glib::Object::builder()
            .property("id", id)
            .property("app_id", app_id)
            .build()
    }
}
