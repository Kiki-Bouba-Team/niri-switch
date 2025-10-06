/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
use glib::Properties;
use glib::prelude::*;
use glib::subclass::prelude::*;
use std::cell::{Cell, RefCell};

/// GObject for holding information about a window
#[derive(Properties, Default)]
#[properties(wrapper_type = super::WindowInfo)]
pub struct WindowInfo {
    #[property(get, set)]
    id: Cell<u64>,

    #[property(get, set)]
    app_name: RefCell<String>,

    #[property(get, set)]
    app_icon: RefCell<Option<gio::Icon>>,
}

#[glib::derived_properties]
impl ObjectImpl for WindowInfo {}

#[glib::object_subclass]
impl ObjectSubclass for WindowInfo {
    const NAME: &'static str = "WindowInfo";
    type Type = super::WindowInfo;
}
