/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
mod imp;

glib::wrapper! {
    pub struct WindowInfo(ObjectSubclass<imp::WindowInfo>);
}

/// GObject for holding information about a window
impl WindowInfo {
    pub fn new(id: u64, app_id: &str) -> Self {
        glib::Object::builder()
            .property("id", id)
            .property("app_id", app_id)
            .build()
    }
}
