/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */
use gio::prelude::{AppInfoExt, IconExt};

/// Stores information about installed aps retrieved from gio
pub struct AppDatabase {
    app_list: Vec<AppInfo>,
}

impl AppDatabase {
    pub fn new() -> Self {
        /* Get information about currently installed applications */
        let app_list = gio::AppInfo::all().iter().map(AppInfo::from).collect();
        Self { app_list }
    }

    pub fn get_app_info(&self, app_id: &String) -> Option<AppInfo> {
        /* Good to try matching the id directly first, for some reason it sometimes
         * works better then the gio method. The gio algorithm is not described
         * anywhere so hard to know why that happens */
        let display_name = self.get_app_info_from_cache(&format!("{app_id}.desktop"));
        if let Some(name) = display_name {
            return Some(name);
        }

        /* Get desktop app names matched to the requested string. The matches come
         * sorted according to the quality of match. Matches with the same quality
         * are put in the same array. The best ones are at the beginning */
        let matches = gio::DesktopAppInfo::search(app_id);
        let best_matches = matches.first()?;

        /* If there are multiple best fit results, choose the shortest one.
         * This is just a heuristic that seems to work fine on average */
        let best_match = best_matches.iter().min_by_key(|app_id| app_id.len())?;

        self.get_app_info_from_cache(&best_match.to_string())
    }

    fn get_app_info_from_cache(&self, requested_id: &String) -> Option<AppInfo> {
        for app_info in &self.app_list {
            if let Some(app_id) = &app_info.app_id
                && app_id == requested_id
            {
                return Some(app_info.clone());
            }
        }
        None
    }
}

/// Type safe wrapper around gio::AppInfo
///
/// This is needed, because gio::AppInfo holds some raw pointers,
/// which are generally not safe to share between threads.
#[derive(Clone)]
pub struct AppInfo {
    pub app_id: Option<String>,
    pub display_name: String,
    /* Serialized gio::Icon */
    pub icon: Option<glib::Variant>,
}

impl From<&gio::AppInfo> for AppInfo {
    fn from(app_info: &gio::AppInfo) -> Self {
        let app_id = app_info.id().map(|id| id.to_string());

        let display_name = app_info.display_name().to_string();

        let icon = match app_info.icon() {
            Some(icon) => icon.serialize(),
            None => None,
        };

        Self {
            app_id,
            display_name,
            icon,
        }
    }
}
