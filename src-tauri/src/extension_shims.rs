use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

/// Provides Linux equivalents for macOS-specific APIs used in Raycast extensions
/// This module helps bridge the gap between macOS and Linux for extension compatibility

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShimResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// Translates macOS paths to Linux equivalents
pub struct PathShim;

impl PathShim {
    /// Maps common macOS paths to their Linux equivalents
    pub fn translate_path(macos_path: &str) -> String {
        // Handle /Applications/ paths
        if macos_path.starts_with("/Applications/") {
            // Try to find the application in common Linux locations
            let app_name = macos_path
                .strip_prefix("/Applications/")
                .unwrap_or(macos_path)
                .trim_end_matches(".app")
                .to_lowercase();
            
            // Return the most likely Linux equivalent
            // Extensions will need to use the desktop file system instead
            return format!("/usr/share/applications/{}.desktop", app_name);
        }
        
        // Handle /Library/ paths
        if macos_path.starts_with("/Library/") {
            let rest = macos_path.strip_prefix("/Library/").unwrap_or("");
            return format!("/usr/lib/{}", rest);
        }
        
        // Handle ~/Library/ paths
        if macos_path.starts_with("~/Library/") {
            let rest = macos_path.strip_prefix("~/Library/").unwrap_or("");
            // Map to XDG directories where appropriate
            if rest.starts_with("Application Support/") {
                let app_rest = rest.strip_prefix("Application Support/").unwrap_or("");
                return format!("~/.local/share/{}", app_rest);
            }
            if rest.starts_with("Preferences/") {
                let pref_rest = rest.strip_prefix("Preferences/").unwrap_or("");
                return format!("~/.config/{}", pref_rest);
            }
            return format!("~/.local/lib/{}", rest);
        }
        
        // Handle /Users/ paths
        if macos_path.starts_with("/Users/") {
            return macos_path.replace("/Users/", "/home/");
        }
        
        // Return unchanged if no translation needed
        macos_path.to_string()
    }
    
    /// Expands ~ in paths to the actual home directory
    pub fn expand_home(path: &str) -> PathBuf {
        if path.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(path.strip_prefix("~/").unwrap_or(path));
            }
        }
        PathBuf::from(path)
    }
}

/// Provides shims for AppleScript functionality
pub struct AppleScriptShim;

impl AppleScriptShim {
    /// Attempts to translate and execute common AppleScript commands
    pub fn run_apple_script(script: &str) -> ShimResult {
        // Parse common AppleScript patterns and translate to Linux equivalents
        
        // Pattern: tell application "AppName" to activate
        if let Some(app_name) = Self::extract_activate_app(script) {
            return Self::activate_application(&app_name);
        }
        
        // Pattern: tell application "AppName" to quit
        if let Some(app_name) = Self::extract_quit_app(script) {
            return Self::quit_application(&app_name);
        }
        
        // Pattern: display notification
        if let Some((title, message)) = Self::extract_notification(script) {
            return Self::show_notification(&title, &message);
        }
        
        // Pattern: set volume
        if let Some(volume) = Self::extract_set_volume(script) {
            return Self::set_system_volume(volume);
        }
        
        // If we can't translate, return an error
        ShimResult {
            success: false,
            output: None,
            error: Some(format!(
                "AppleScript not supported on Linux. Script: {}",
                script
            )),
        }
    }
    
    fn extract_activate_app(script: &str) -> Option<String> {
        // Match: tell application "AppName" to activate
        let patterns = [
            r#"tell application "([^"]+)" to activate"#,
            r#"activate application "([^"]+)""#,
        ];
        
        for pattern in &patterns {
            if let Some(caps) = regex::Regex::new(pattern).ok()?.captures(script) {
                return caps.get(1).map(|m| m.as_str().to_string());
            }
        }
        None
    }
    
    fn extract_quit_app(script: &str) -> Option<String> {
        // Match: tell application "AppName" to quit
        let pattern = r#"tell application "([^"]+)" to quit"#;
        regex::Regex::new(pattern)
            .ok()?
            .captures(script)?
            .get(1)
            .map(|m| m.as_str().to_string())
    }
    
    fn extract_notification(script: &str) -> Option<(String, String)> {
        // Match: display notification "message" with title "title"
        let pattern = r#"display notification "([^"]+)"(?:\s+with title "([^"]+)")?"#;
        let caps = regex::Regex::new(pattern).ok()?.captures(script)?;
        
        let message = caps.get(1)?.as_str().to_string();
        let title = caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_else(|| "Notification".to_string());
        
        Some((title, message))
    }
    
    fn extract_set_volume(script: &str) -> Option<i32> {
        // Match: set volume N or set volume output volume N
        let patterns = [
            r"set volume (\d+)",
            r"set volume output volume (\d+)",
        ];
        
        for pattern in &patterns {
            if let Some(caps) = regex::Regex::new(pattern).ok()?.captures(script) {
                return caps.get(1)?.as_str().parse().ok();
            }
        }
        None
    }
    
    fn activate_application(app_name: &str) -> ShimResult {
        // Try to launch the application using the desktop file
        let desktop_name = app_name.to_lowercase();
        
        // Try using gtk-launch (works on most desktop environments)
        let output = Command::new("gtk-launch")
            .arg(&desktop_name)
            .output();
        
        match output {
            Ok(out) if out.status.success() => ShimResult {
                success: true,
                output: Some(format!("Activated application: {}", app_name)),
                error: None,
            },
            _ => {
                // Fallback: try xdg-open
                let fallback = Command::new("xdg-open")
                    .arg(&desktop_name)
                    .output();
                
                match fallback {
                    Ok(out) if out.status.success() => ShimResult {
                        success: true,
                        output: Some(format!("Activated application: {}", app_name)),
                        error: None,
                    },
                    _ => ShimResult {
                        success: false,
                        output: None,
                        error: Some(format!("Failed to activate application: {}", app_name)),
                    },
                }
            }
        }
    }
    
    fn quit_application(app_name: &str) -> ShimResult {
        // Try to quit the application using pkill
        let process_name = app_name.to_lowercase();
        
        let output = Command::new("pkill")
            .arg("-f")
            .arg(&process_name)
            .output();
        
        match output {
            Ok(out) if out.status.success() => ShimResult {
                success: true,
                output: Some(format!("Quit application: {}", app_name)),
                error: None,
            },
            _ => ShimResult {
                success: false,
                output: None,
                error: Some(format!("Failed to quit application: {}", app_name)),
            },
        }
    }
    
    fn show_notification(title: &str, message: &str) -> ShimResult {
        // Use notify-send for freedesktop notifications
        let output = Command::new("notify-send")
            .arg(title)
            .arg(message)
            .output();
        
        match output {
            Ok(out) if out.status.success() => ShimResult {
                success: true,
                output: Some("Notification sent".to_string()),
                error: None,
            },
            _ => ShimResult {
                success: false,
                output: None,
                error: Some("Failed to send notification".to_string()),
            },
        }
    }
    
    fn set_system_volume(volume: i32) -> ShimResult {
        // Clamp volume to 0-100
        let vol = volume.clamp(0, 100);
        
        // Try using pactl (PulseAudio/PipeWire)
        let output = Command::new("pactl")
            .arg("set-sink-volume")
            .arg("@DEFAULT_SINK@")
            .arg(format!("{}%", vol))
            .output();
        
        match output {
            Ok(out) if out.status.success() => ShimResult {
                success: true,
                output: Some(format!("Set volume to {}%", vol)),
                error: None,
            },
            _ => {
                // Fallback: try amixer (ALSA)
                let fallback = Command::new("amixer")
                    .arg("set")
                    .arg("Master")
                    .arg(format!("{}%", vol))
                    .output();
                
                match fallback {
                    Ok(out) if out.status.success() => ShimResult {
                        success: true,
                        output: Some(format!("Set volume to {}%", vol)),
                        error: None,
                    },
                    _ => ShimResult {
                        success: false,
                        output: None,
                        error: Some("Failed to set volume".to_string()),
                    },
                }
            }
        }
    }
}

/// System API shims for common macOS system operations
pub struct SystemShim;

impl SystemShim {
    /// Get system information that might be requested by extensions
    pub fn get_system_info() -> HashMap<String, String> {
        let mut info = HashMap::new();
        
        // Platform
        info.insert("platform".to_string(), "linux".to_string());
        
        // Architecture
        if let Ok(output) = Command::new("uname").arg("-m").output() {
            if let Ok(arch) = String::from_utf8(output.stdout) {
                info.insert("arch".to_string(), arch.trim().to_string());
            }
        }
        
        // Hostname
        if let Ok(output) = Command::new("hostname").output() {
            if let Ok(hostname) = String::from_utf8(output.stdout) {
                info.insert("hostname".to_string(), hostname.trim().to_string());
            }
        }
        
        // Desktop environment
        if let Ok(de) = std::env::var("XDG_CURRENT_DESKTOP") {
            info.insert("desktop_environment".to_string(), de);
        }
        
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_path_translation_applications() {
        assert_eq!(
            PathShim::translate_path("/Applications/Safari.app"),
            "/usr/share/applications/safari.desktop"
        );
    }
    
    #[test]
    fn test_path_translation_library() {
        assert_eq!(
            PathShim::translate_path("/Library/Frameworks/Something"),
            "/usr/lib/Frameworks/Something"
        );
    }
    
    #[test]
    fn test_path_translation_users() {
        assert_eq!(
            PathShim::translate_path("/Users/john/Documents"),
            "/home/john/Documents"
        );
    }
    
    #[test]
    fn test_path_translation_user_library() {
        assert_eq!(
            PathShim::translate_path("~/Library/Application Support/MyApp"),
            "~/.local/share/MyApp"
        );
    }
    
    #[test]
    fn test_extract_activate_app() {
        let script = r#"tell application "Safari" to activate"#;
        assert_eq!(
            AppleScriptShim::extract_activate_app(script),
            Some("Safari".to_string())
        );
    }
    
    #[test]
    fn test_extract_notification() {
        let script = r#"display notification "Hello World" with title "Test""#;
        assert_eq!(
            AppleScriptShim::extract_notification(script),
            Some(("Test".to_string(), "Hello World".to_string()))
        );
    }
}
