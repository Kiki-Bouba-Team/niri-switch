/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
mod imp;

use glib::subclass::types::ObjectSubclassIsExt;
use gtk4::glib;

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
        if let Some(label) = imp.title.borrow_mut().take() {
            label.set_label(&format!("{}: {}", window_info.id(), window_info.app_id()));
        }
    }
}
