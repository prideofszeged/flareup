use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JumpModeConfig {
    pub enabled: bool,
    pub max_results: usize,
    pub search_hidden: bool,
    pub exclude_patterns: Vec<String>,
}

impl Default for JumpModeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_results: 50,
            search_hidden: false,
            exclude_patterns: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".cache".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[cfg(test)]
pub struct JumpModeState {
    pub last_query: Option<String>,
    pub last_results: Vec<String>,
    pub search_count: u64,
}

#[cfg(test)]
impl Default for JumpModeState {
    fn default() -> Self {
        Self {
            last_query: None,
            last_results: Vec::new(),
            search_count: 0,
        }
    }
}

#[cfg(test)]
impl JumpModeState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_search(&mut self, query: String, results: Vec<String>) {
        self.last_query = Some(query);
        self.last_results = results;
        self.search_count += 1;
    }
}

/// Check if fd is available on the system
pub fn is_fd_available() -> bool {
    Command::new("fd")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Execute fd search with fuzzy query
pub fn execute_fd_search(
    query: &str,
    config: &JumpModeConfig,
    search_path: Option<&str>,
) -> Result<Vec<String>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    // Check if fd is available
    if !is_fd_available() {
        return Err("fd command not found. Please install fd-find.".to_string());
    }

    let search_dir = match search_path { Some(p) => p.to_string(), None => std::env::var("HOME").unwrap_or_else(|_| "/".to_string()) };

    debug!(
        "Executing fd search: query='{}', path='{}'",
        query, search_dir
    );

    let mut cmd = Command::new("fd");

    // Basic options
    cmd.arg("--type").arg("f"); // Files only
    cmd.arg("-i"); // Case insensitive

    // Hidden files option
    if config.search_hidden {
        cmd.arg("--hidden");
    }

    // Exclude patterns
    for pattern in &config.exclude_patterns {
        cmd.arg("--exclude").arg(pattern);
    }

    // Add the search query
    cmd.arg(query);

    // Add search path
    cmd.arg(search_dir);

    // Execute command
    match cmd.output() {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("fd search failed: {}", stderr);
                return Err(format!("Search failed: {}", stderr));
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut results: Vec<String> = stdout
                .lines()
                .take(config.max_results)
                .map(|line| {
                    // Replace home directory with ~
                    if let Ok(home) = std::env::var("HOME") {
                        line.replace(&home, "~")
                    } else {
                        line.to_string()
                    }
                })
                .collect();

            debug!("fd search returned {} results", results.len());

            // Sort by path length (shorter paths first - likely more relevant)
            results.sort_by_key(|a| a.len());

            Ok(results)
        }
        Err(e) => {
            error!("Failed to execute fd command: {}", e);
            Err(format!("Failed to execute search: {}", e))
        }
    }
}

/// Open a file in the configured editor
pub fn open_in_editor(path: &str) -> Result<(), String> {
    // Expand ~ to home directory
    let expanded_path = if path.starts_with('~') {
        if let Ok(home) = std::env::var("HOME") {
            path.replacen("~", &home, 1)
        } else {
            path.to_string()
        }
    } else {
        path.to_string()
    };

    // Check if file exists
    let file_path = PathBuf::from(&expanded_path);
    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    // Get editor from environment variable
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
        warn!("$EDITOR not set, using xdg-open as fallback");
        "xdg-open".to_string()
    });

    info!("Opening file '{}' with editor '{}'", path, editor);

    // Execute editor command
    match Command::new(&editor).arg(&expanded_path).spawn() {
        Ok(_) => {
            debug!("Successfully spawned editor process");
            Ok(())
        }
        Err(e) => {
            error!("Failed to open file in editor: {}", e);
            Err(format!("Failed to open file: {}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jump_mode_config_default() {
        let config = JumpModeConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_results, 50);
        assert!(!config.search_hidden);
        assert_eq!(config.exclude_patterns.len(), 4);
    }

    #[test]
    fn test_jump_mode_state_update() {
        let mut state = JumpModeState::new();
        assert_eq!(state.search_count, 0);

        state.update_search("test".to_string(), vec!["file1.rs".to_string()]);
        assert_eq!(state.search_count, 1);
        assert_eq!(state.last_query, Some("test".to_string()));
        assert_eq!(state.last_results.len(), 1);
    }

    #[test]
    fn test_is_fd_available() {
        // This test will pass/fail depending on whether fd is installed
        let available = is_fd_available();
        println!("fd available: {}", available);
    }
}
