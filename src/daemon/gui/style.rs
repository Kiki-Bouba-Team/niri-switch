/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
use std::{env, path::PathBuf};

const APP_CONFIG_DIR: &str = "niri-switch";
const STYLESHEET_FILENAME: &str = "style.css";

/// Applies the style sheet to the window
pub fn load_css() {
    let css_provider = gtk4::CssProvider::new();

    if !try_loading_user_provided_css(&css_provider) {
        /* If no custom css provided, fallback to the embeded file */
        css_provider.load_from_string(include_str!("style.css"));
    }

    gtk4::style_context_add_provider_for_display(
        &gdk4::Display::default().expect("Could not connect to the default display"),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

/// Try loading custom css stylesheet provided by user into css provider
///
/// It will first attempt to load stylesheet from `$XDG_CONFIG_HOME/niri-switch/style.css`,
/// if unsuccessful, it will try to load styles from `$HOME/.config/niri-switch/style.css`.
fn try_loading_user_provided_css(css_provider: &gtk4::CssProvider) -> bool {
    /* First try to retrieve stylesheet from XDG_CONFIG_HOME/niri-switch */
    if let Ok(config_path) = env::var("XDG_CONFIG_HOME") {
        let stylesheet_path = PathBuf::from(config_path)
            .join(APP_CONFIG_DIR)
            .join(STYLESHEET_FILENAME);
        if stylesheet_path.exists() {
            /* Stylesheet found, load it into the provider */
            let css_file = gio::File::for_path(stylesheet_path);
            css_provider.load_from_file(&css_file);
            return true;
        }
    }

    /* No luck with XDG_CONFIG_HOME, try $HOME/.config/niri-switch instead */
    if let Ok(home_path) = env::var("HOME") {
        let stylesheet_path = PathBuf::from(home_path)
            .join(".config")
            .join(APP_CONFIG_DIR)
            .join(STYLESHEET_FILENAME);
        if stylesheet_path.exists() {
            /* Stylesheet found, load it into the provider */
            let css_file = gio::File::for_path(stylesheet_path);
            css_provider.load_from_file(&css_file);
            return true;
        }
    }

    /* Custom stylesheet not found */
    false
}
