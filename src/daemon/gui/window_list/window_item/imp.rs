/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
use glib::subclass::InitializingObject;
use gtk4::subclass::prelude::*;

/* Here we create custom widget for displaying window info by
 * subclassing gtk4::Box. The widget layout will be loaded from
 * the window_item.ui file */
#[derive(Debug, Default, gtk4::CompositeTemplate)]
#[template(resource = "/org/kikibouba/niriswitch/window_list/window_item/window_item.ui")]
pub struct WindowItem {
    #[template_child]
    pub app_name: TemplateChild<gtk4::Label>,

    #[template_child]
    pub title: TemplateChild<gtk4::Label>,

    #[template_child]
    pub icon: TemplateChild<gtk4::Image>,
}

#[glib::object_subclass]
impl ObjectSubclass for WindowItem {
    const NAME: &'static str = "WindowItem";
    type Type = super::WindowItem;
    type ParentType = gtk4::Box;

    fn class_init(class: &mut Self::Class) {
        class.bind_template();
        class.set_css_name("window-item-box");
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for WindowItem {}
impl WidgetImpl for WindowItem {}
impl BoxImpl for WindowItem {}
