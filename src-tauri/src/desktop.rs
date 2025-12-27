use crate::{app::App, error::AppError};
use freedesktop_file_parser::{parse, EntryType};
use rayon::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

pub struct DesktopFileManager;

impl DesktopFileManager {
    pub fn get_app_directories() -> Vec<PathBuf> {
        let mut app_dirs = vec![
            PathBuf::from("/usr/share/applications"),
            PathBuf::from("/usr/local/share/applications"),
        ];

        if let Ok(home_dir) = env::var("HOME") {
            app_dirs.push(PathBuf::from(home_dir).join(".local/share/applications"));
        }
        app_dirs
    }

    pub fn find_desktop_files(path: &Path) -> Vec<PathBuf> {
        let mut desktop_files = Vec::new();
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    desktop_files.extend(Self::find_desktop_files(&path));
                } else if path.extension().map_or(false, |ext| ext == "desktop") {
                    desktop_files.push(path);
                }
            }
        }
        desktop_files
    }

    pub fn scan_and_parse_apps() -> Result<(Vec<App>, HashMap<PathBuf, SystemTime>), AppError> {
        let app_dirs = Self::get_app_directories();
        let desktop_files: Vec<PathBuf> = app_dirs
            .iter()
            .filter(|dir| dir.exists())
            .flat_map(|dir| Self::find_desktop_files(dir))
            .collect();

        let apps: Vec<App> = desktop_files
            .par_iter()
            .filter_map(|file_path| Self::parse_desktop_file(file_path))
            .collect();

        let unique_apps = Self::deduplicate_and_sort_apps(apps);

        let dir_mod_times = Self::get_directory_modification_times(app_dirs)?;

        Ok((unique_apps, dir_mod_times))
    }

    fn parse_desktop_file(file_path: &Path) -> Option<App> {
        let content = fs::read_to_string(file_path).ok()?;
        let desktop_file = parse(&content).ok()?;

        if desktop_file.entry.hidden.unwrap_or(false)
            || desktop_file.entry.no_display.unwrap_or(false)
        {
            return None;
        }

        // Check OnlyShowIn and NotShowIn to filter by desktop environment
        if !Self::should_show_in_current_desktop(
            &desktop_file.entry.only_show_in,
            &desktop_file.entry.not_show_in,
        ) {
            return None;
        }

        if let EntryType::Application(app_fields) = desktop_file.entry.entry_type {
            if app_fields.exec.is_some() && !desktop_file.entry.name.default.is_empty() {
                return Some(
                    App::new(desktop_file.entry.name.default)
                        .with_comment(desktop_file.entry.comment.map(|lc| lc.default))
                        .with_exec(app_fields.exec)
                        .with_icon_path(
                            desktop_file
                                .entry
                                .icon
                                .and_then(|ic| ic.get_icon_path())
                                .and_then(|p| p.to_str().map(String::from)),
                        ),
                );
            }
        }
        None
    }

    /// Check if an app should be shown in the current desktop environment
    fn should_show_in_current_desktop(
        only_show_in: &Option<Vec<String>>,
        not_show_in: &Option<Vec<String>>,
    ) -> bool {
        // Get current desktop environment(s)
        let current_desktops = Self::get_current_desktops();

        // If OnlyShowIn is specified, the app should only show if current desktop matches
        if let Some(allowed_desktops) = only_show_in {
            if !allowed_desktops.is_empty() {
                // Check if any current desktop matches any allowed desktop
                let matches = current_desktops.iter().any(|current| {
                    allowed_desktops
                        .iter()
                        .any(|allowed| current.eq_ignore_ascii_case(allowed))
                });
                if !matches {
                    return false;
                }
            }
        }

        // If NotShowIn is specified, the app should NOT show if current desktop matches
        if let Some(excluded_desktops) = not_show_in {
            if !excluded_desktops.is_empty() {
                let excluded = current_desktops.iter().any(|current| {
                    excluded_desktops
                        .iter()
                        .any(|excluded| current.eq_ignore_ascii_case(excluded))
                });
                if excluded {
                    return false;
                }
            }
        }

        true
    }

    /// Get the current desktop environment(s)
    fn get_current_desktops() -> Vec<String> {
        // XDG_CURRENT_DESKTOP can contain multiple values separated by colons
        // e.g., "Unity:GNOME" or "X-Cinnamon"
        if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
            return desktop.split(':').map(|s| s.to_string()).collect();
        }

        // Fallback to DESKTOP_SESSION
        if let Ok(session) = env::var("DESKTOP_SESSION") {
            return vec![session];
        }

        // Unknown desktop - show all apps
        vec![]
    }

    fn deduplicate_and_sort_apps(apps: Vec<App>) -> Vec<App> {
        let mut unique_apps = Vec::new();
        let mut seen_app_names = HashSet::new();

        for app in apps {
            if seen_app_names.insert(app.name.clone()) {
                unique_apps.push(app);
            }
        }

        unique_apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        unique_apps
    }

    fn get_directory_modification_times(
        app_dirs: Vec<PathBuf>,
    ) -> Result<HashMap<PathBuf, SystemTime>, AppError> {
        Ok(app_dirs
            .into_iter()
            .filter_map(|dir| {
                fs::metadata(&dir)
                    .and_then(|m| m.modified())
                    .ok()
                    .map(|mod_time| (dir, mod_time))
            })
            .collect())
    }
}
