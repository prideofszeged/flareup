use super::{manager::FileSearchManager, types::IndexedFile};
use std::{env, path::PathBuf, time::SystemTime};
use tauri::{AppHandle, Manager};
use walkdir::{DirEntry, WalkDir};

pub async fn build_initial_index(app_handle: AppHandle) {
    println!("Starting initial file index build.");
    let manager = app_handle.state::<FileSearchManager>();
    let home_dir = match env::var("HOME") {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Failed to get home directory: {}", e);
            return;
        }
    };

    // Index only specific directories, not entire home
    let index_dirs = [
        "Documents",
        "Downloads",
        "Desktop",
        "Pictures",
        "Videos",
        "Music",
        "Projects",
        "Code",
        "dev",
        "workspace",
    ];

    // Load all existing file timestamps in a single query to avoid N+1 problem
    let existing_files = match manager.get_all_file_timestamps() {
        Ok(timestamps) => timestamps,
        Err(e) => {
            eprintln!("Failed to load existing file timestamps: {}", e);
            std::collections::HashMap::new()
        }
    };

    let mut indexed_count = 0;
    for dir_name in &index_dirs {
        let dir_path = PathBuf::from(&home_dir).join(dir_name);
        if !dir_path.exists() || !dir_path.is_dir() {
            continue;
        }

        println!("Indexing {}...", dir_path.display());
        let walker = WalkDir::new(&dir_path).into_iter();
        for entry in walker.filter_entry(|e| !is_hidden(e) && !is_excluded(e)) {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    eprintln!("Error walking directory: {}", e);
                    continue;
                }
            };

            let path = entry.path();
            let metadata = match entry.metadata() {
                Ok(meta) => meta,
                Err(_) => continue,
            };

            let last_modified_secs = metadata
                .modified()
                .unwrap_or(SystemTime::UNIX_EPOCH)
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            // Use in-memory HashMap lookup instead of database query
            if let Some(&indexed_time) = existing_files.get(&path.to_string_lossy().to_string()) {
                if indexed_time >= last_modified_secs {
                    if path.is_dir() {
                        // continue to walk children
                    } else {
                        // skip this file
                        continue;
                    }
                }
            }

            let file_type = if metadata.is_dir() {
                "directory".to_string()
            } else if metadata.is_file() {
                "file".to_string()
            } else {
                continue;
            };

            let indexed_file = IndexedFile {
                path: path.to_string_lossy().to_string(),
                name: entry.file_name().to_string_lossy().to_string(),
                parent_path: path
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
                file_type,
                last_modified: last_modified_secs,
            };

            if let Err(e) = manager.add_file(&indexed_file) {
                eprintln!("Failed to add file to index: {:?}", e);
            } else {
                indexed_count += 1;
            }
        }
    }

    println!(
        "✅ Finished initial file index build. Indexed {} files.",
        indexed_count
    );
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn is_excluded(entry: &DirEntry) -> bool {
    let path = entry.path();
    let excluded_dirs = [
        "node_modules",
        ".git",
        ".svn",
        "target",
        "build",
        ".vscode",
        ".idea",
        "__pycache__",
        ".pytest_cache",
        ".mypy_cache",
        ".cache",
        ".local/share/Trash",
        ".gradle",
        ".wine",
        ".wine-qoder",
        ".npm",
        ".cargo",
        ".rustup",
        ".pnpm-store",
        "venv",
        ".venv",
        "Library",
        "Application Support",
        "AppData",
    ];
    path.components().any(|component| {
        if let Some(name) = component.as_os_str().to_str() {
            excluded_dirs
                .iter()
                .any(|&excluded| name == excluded || name.starts_with(&format!("{}.", excluded)))
        } else {
            false
        }
    })
}
