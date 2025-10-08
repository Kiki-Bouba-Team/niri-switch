/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
mod imp;

glib::wrapper! {
    pub struct WindowInfo(ObjectSubclass<imp::WindowInfo>);
}

/// GObject for holding information about a window
impl WindowInfo {
    pub fn new(id: u64, title: &String, app_name: &String,  app_icon: Option<gio::Icon>) -> Self {
        glib::Object::builder()
            .property("id", id)
            .property("title", title)
            .property("app_name", app_name)
            .property("app_icon", app_icon)
            .build()
    }
}
