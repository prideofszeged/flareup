use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Registry of macOS tools and their Linux equivalents
/// Mason-like system for automatic tool installation and shimming

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LinuxPackage {
    /// Install via apt-get (Debian/Ubuntu)
    Apt { package: String },
    /// Install via dnf (Fedora/RHEL)
    Dnf { package: String },
    /// Install via pacman (Arch)
    Pacman { package: String },
    /// Install via flatpak
    Flatpak { id: String },
    /// Download and install binary
    Binary { url: String, install_script: String },
    /// Built-in Rust implementation (no installation needed)
    Builtin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShimType {
    /// Run Linux command directly (1:1 mapping)
    DirectExec,
    /// Substitute in PATH with wrapper script
    PathSubstitute,
    /// Generate a wrapper script
    WrapperScript,
    /// Use our Rust shim implementation
    RustShim,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMapping {
    /// macOS tool name (e.g., "pbcopy", "speedtest")
    pub macos_tool: String,
    /// Linux package to install
    pub linux_package: LinuxPackage,
    /// Command to test if tool is installed
    pub test_command: String,
    /// How to shim this tool
    pub shim_type: ShimType,
    /// Description for UI
    pub description: Option<String>,
}

impl ToolMapping {
    pub fn new(
        macos_tool: impl Into<String>,
        linux_package: LinuxPackage,
        test_command: impl Into<String>,
        shim_type: ShimType,
    ) -> Self {
        Self {
            macos_tool: macos_tool.into(),
            linux_package,
            test_command: test_command.into(),
            shim_type,
            description: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Registry of common macOS tools and their Linux equivalents
pub struct ToolRegistry {
    tools: HashMap<String, ToolMapping>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };

        // Register common tools
        registry.register_default_tools();
        registry
    }

    fn register_default_tools(&mut self) {
        // Clipboard tools
        self.add(
            ToolMapping::new(
                "pbcopy",
                LinuxPackage::Apt {
                    package: "xclip".to_string(),
                },
                "xclip -version",
                ShimType::WrapperScript,
            )
            .with_description("Clipboard copy"),
        );

        self.add(
            ToolMapping::new(
                "pbpaste",
                LinuxPackage::Apt {
                    package: "xclip".to_string(),
                },
                "xclip -version",
                ShimType::WrapperScript,
            )
            .with_description("Clipboard paste"),
        );

        // Open command
        self.add(
            ToolMapping::new(
                "open",
                LinuxPackage::Builtin, // xdg-open is usually preinstalled
                "xdg-open --version",
                ShimType::DirectExec,
            )
            .with_description("Open files/URLs"),
        );

        // AppleScript
        self.add(
            ToolMapping::new(
                "osascript",
                LinuxPackage::Builtin,
                "echo 'builtin'",
                ShimType::RustShim,
            )
            .with_description("AppleScript execution (shimmed)"),
        );

        // Text-to-speech
        self.add(
            ToolMapping::new(
                "say",
                LinuxPackage::Apt {
                    package: "espeak".to_string(),
                },
                "espeak --version",
                ShimType::WrapperScript,
            )
            .with_description("Text-to-speech"),
        );

        // Prevent sleep
        self.add(
            ToolMapping::new(
                "caffeinate",
                LinuxPackage::Builtin,
                "systemd-inhibit --version",
                ShimType::WrapperScript,
            )
            .with_description("Prevent system sleep"),
        );

        // Speedtest (common in extensions)
        self.add(
            ToolMapping::new(
                "speedtest",
                LinuxPackage::Binary {
                    url: "https://install.speedtest.net/app/cli/ookla-speedtest-1.2.0-linux-x86_64.tgz".to_string(),
                    install_script: "tar xzf - -C ~/.local/bin".to_string(),
                },
                "speedtest --version",
                ShimType::DirectExec,
            )
            .with_description("Network speed testing"),
        );

        // JSON processor (very common)
        self.add(
            ToolMapping::new(
                "jq",
                LinuxPackage::Apt {
                    package: "jq".to_string(),
                },
                "jq --version",
                ShimType::DirectExec,
            )
            .with_description("JSON processor"),
        );

        // Image manipulation
        self.add(
            ToolMapping::new(
                "sips",
                LinuxPackage::Apt {
                    package: "imagemagick".to_string(),
                },
                "magick --version",
                ShimType::WrapperScript,
            )
            .with_description("Image processing"),
        );

        // QL tools (Quick Look)
        self.add(
            ToolMapping::new(
                "qlmanage",
                LinuxPackage::Builtin,
                "echo 'not supported'",
                ShimType::RustShim,
            )
            .with_description("Quick Look (limited support)"),
        );
    }

    pub fn add(&mut self, mapping: ToolMapping) {
        self.tools.insert(mapping.macos_tool.clone(), mapping);
    }

    pub fn get(&self, tool: &str) -> Option<&ToolMapping> {
        self.tools.get(tool)
    }

    pub fn all(&self) -> Vec<&ToolMapping> {
        self.tools.values().collect()
    }

    pub fn find_tools_in_code(&self, code: &str) -> Vec<&ToolMapping> {
        self.tools
            .values()
            .filter(|mapping| {
                // Look for shell command patterns
                let patterns = [
                    format!("\"{}\"", mapping.macos_tool),
                    format!("'{}'", mapping.macos_tool),
                    format!("`{}`", mapping.macos_tool),
                    format!("exec('{}'", mapping.macos_tool),
                    format!("spawn('{}'", mapping.macos_tool),
                ];
                patterns.iter().any(|p| code.contains(p))
            })
            .collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect Linux distribution
#[derive(Debug, Clone, PartialEq)]
pub enum Distro {
    Debian,
    Ubuntu,
    Fedora,
    Arch,
    Unknown,
}

pub fn detect_distro() -> Distro {
    if let Ok(os_release) = fs::read_to_string("/etc/os-release") {
        let lower = os_release.to_lowercase();
        if lower.contains("ubuntu") {
            return Distro::Ubuntu;
        } else if lower.contains("debian") {
            return Distro::Debian;
        } else if lower.contains("fedora") {
            return Distro::Fedora;
        } else if lower.contains("arch") {
            return Distro::Arch;
        }
    }
    Distro::Unknown
}

/// Check if a tool is installed
pub fn is_tool_installed(test_command: &str) -> bool {
    let parts: Vec<&str> = test_command.split_whitespace().collect();
    if parts.is_empty() {
        return false;
    }

    Command::new(parts[0])
        .args(&parts[1..])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get the shim directory path
pub fn get_shim_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("flareup")
        .join("shims")
}

/// Generate a wrapper script for a tool
pub fn generate_wrapper_script(mapping: &ToolMapping) -> Option<String> {
    match mapping.macos_tool.as_str() {
        "pbcopy" => Some(
            r#"#!/bin/bash
# Auto-generated shim for pbcopy

if command -v wl-copy &> /dev/null; then
    wl-copy "$@"
elif command -v xclip &> /dev/null; then
    xclip -selection clipboard -i "$@"
else
    echo "Error: No clipboard tool found. Install wl-copy (Wayland) or xclip (X11)" >&2
    exit 1
fi
"#
            .to_string(),
        ),
        "pbpaste" => Some(
            r#"#!/bin/bash
# Auto-generated shim for pbpaste

if command -v wl-paste &> /dev/null; then
    wl-paste "$@"
elif command -v xclip &> /dev/null; then
    xclip -selection clipboard -o "$@"
else
    echo "Error: No clipboard tool found. Install wl-paste (Wayland) or xclip (X11)" >&2
    exit 1
fi
"#
            .to_string(),
        ),
        "say" => Some(
            r#"#!/bin/bash
# Auto-generated shim for say (text-to-speech)

if command -v espeak &> /dev/null; then
    # espeak uses different syntax than say
    echo "$@" | espeak
elif command -v festival &> /dev/null; then
    echo "$@" | festival --tts
else
    echo "Error: No TTS tool found. Install espeak or festival" >&2
    exit 1
fi
"#
            .to_string(),
        ),
        "caffeinate" => Some(
            r#"#!/bin/bash
# Auto-generated shim for caffeinate

if command -v systemd-inhibit &> /dev/null; then
    # Run the command with sleep inhibition
    systemd-inhibit --what=idle:sleep "$@"
else
    # Fallback: just run the command
    "$@"
fi
"#
            .to_string(),
        ),
        "sips" => Some(
            r#"#!/bin/bash
# Auto-generated shim for sips (image processing)
# Maps common sips operations to ImageMagick

echo "Error: sips shim not yet implemented. Use 'convert' (ImageMagick) directly" >&2
exit 1
"#
            .to_string(),
        ),
        _ => None,
    }
}

/// Create shim directory and install wrapper scripts
pub fn install_shims(mappings: &[&ToolMapping]) -> Result<(), String> {
    let shim_dir = get_shim_dir();
    fs::create_dir_all(&shim_dir).map_err(|e| format!("Failed to create shim directory: {}", e))?;

    for mapping in mappings {
        if mapping.shim_type == ShimType::WrapperScript {
            if let Some(script) = generate_wrapper_script(mapping) {
                let script_path = shim_dir.join(&mapping.macos_tool);
                fs::write(&script_path, script)
                    .map_err(|e| format!("Failed to write shim script: {}", e))?;

                // Make executable
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&script_path)
                        .map_err(|e| format!("Failed to get permissions: {}", e))?
                        .permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&script_path, perms)
                        .map_err(|e| format!("Failed to set permissions: {}", e))?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ToolRegistry::new();
        assert!(registry.get("pbcopy").is_some());
        assert!(registry.get("osascript").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_find_tools_in_code() {
        let registry = ToolRegistry::new();
        let code = r#"
            const result = exec('pbcopy');
            spawn('jq', ['.name']);
        "#;

        let found = registry.find_tools_in_code(code);
        assert!(found.len() >= 2);
        assert!(found.iter().any(|t| t.macos_tool == "pbcopy"));
        assert!(found.iter().any(|t| t.macos_tool == "jq"));
    }

    #[test]
    fn test_detect_distro() {
        let distro = detect_distro();
        // Just ensure it doesn't panic
        assert!(matches!(
            distro,
            Distro::Debian | Distro::Ubuntu | Distro::Fedora | Distro::Arch | Distro::Unknown
        ));
    }

    #[test]
    fn test_wrapper_generation() {
        let mapping = ToolMapping::new(
            "pbcopy",
            LinuxPackage::Apt {
                package: "xclip".to_string(),
            },
            "xclip -version",
            ShimType::WrapperScript,
        );

        let script = generate_wrapper_script(&mapping);
        assert!(script.is_some());
        let script = script.unwrap();
        assert!(script.contains("#!/bin/bash"));
        assert!(script.contains("xclip"));
    }

    #[test]
    fn test_shim_dir_path() {
        let shim_dir = get_shim_dir();
        assert!(shim_dir.to_string_lossy().contains("flareup"));
        assert!(shim_dir.to_string_lossy().contains("shims"));
    }
}
