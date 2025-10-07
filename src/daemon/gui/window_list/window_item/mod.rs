/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
mod imp;

use gtk4::glib;
use gtk4::subclass::prelude::*;

/* Here we create custom widget for displaying window info by
 * subclassing gtk4::Box */
glib::wrapper! {
    pub struct WindowItem(ObjectSubclass<imp::WindowItem>)
        @extends gtk4::Widget, gtk4::Box,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for WindowItem {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl WindowItem {
    /// Fill the widgets based on WindowInfo
    pub fn set_window_info(&self, window_info: super::window_info::WindowInfo) {
        let imp = self.imp();

        imp.title.set_label(&window_info.app_name());

        match window_info.app_icon() {
            Some(gicon) => {
                imp.icon.set_from_gicon(&gicon);
            }
            None => {
                imp.icon.set_icon_name(Some("application-x-executable"));
            }
        };
    }
}
