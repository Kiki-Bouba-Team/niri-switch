/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
use gtk4::{prelude::WidgetExt, subclass::prelude::*};
use std::cell::RefCell;

/* Here we create custom widget for displaying window info by
 * subclassing gtk4::Box */
#[derive(Debug, Default)]
pub struct WindowItem {
    pub title: RefCell<Option<gtk4::Label>>,
}

#[glib::object_subclass]
impl ObjectSubclass for WindowItem {
    const NAME: &'static str = "WindowItem";
    type Type = super::WindowItem;
    type ParentType = gtk4::Box;

    fn class_init(class: &mut Self::Class) {
        class.set_css_name("window-item-box");
    }
}

impl ObjectImpl for WindowItem {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();

        /* Create an empty label for displaying the title */
        let label = gtk4::Label::builder().css_name("window-item-label").build();
        label.set_parent(&*obj);

        /* Save a referance to the label */
        self.title.replace(Some(label));
    }

    fn dispose(&self) {
        /* Destructor needs to unparent children manually */
        if let Some(label) = self.title.borrow_mut().take() {
            label.unparent();
        }
    }
}

impl WidgetImpl for WindowItem {}
impl BoxImpl for WindowItem {}
