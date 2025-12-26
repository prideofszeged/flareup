//! AI Tool Use Module
//!
//! Provides built-in tools for AI function calling:
//! - File operations (read, write, list, search, delete)
//! - System information
//! - Shell command execution
//! - Clipboard operations

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

/// Maximum file size that can be read (5MB)
pub const MAX_FILE_READ_SIZE: usize = 5 * 1024 * 1024;

/// Tool safety classification for confirmation dialogs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolSafety {
    /// Auto-execute without confirmation (read, list, search, system_info)
    Safe,
    /// Requires user confirmation (write, delete, execute, clipboard)
    Dangerous,
}

/// Tool definition for OpenAI function calling format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

/// A tool call request from the AI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

/// Result of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResult {
    pub tool_call_id: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

/// Built-in tool names
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinTool {
    ReadFile,
    WriteFile,
    ListDirectory,
    SearchFiles,
    DeleteFile,
    GetSystemInfo,
    RunCommand,
    ReadClipboard,
    WriteClipboard,
}

impl BuiltinTool {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ReadFile => "read_file",
            Self::WriteFile => "write_file",
            Self::ListDirectory => "list_directory",
            Self::SearchFiles => "search_files",
            Self::DeleteFile => "delete_file",
            Self::GetSystemInfo => "get_system_info",
            Self::RunCommand => "run_command",
            Self::ReadClipboard => "read_clipboard",
            Self::WriteClipboard => "write_clipboard",
        }
    }

    pub fn safety(&self) -> ToolSafety {
        match self {
            Self::ReadFile | Self::ListDirectory | Self::SearchFiles | Self::GetSystemInfo => {
                ToolSafety::Safe
            }
            Self::WriteFile
            | Self::DeleteFile
            | Self::RunCommand
            | Self::ReadClipboard
            | Self::WriteClipboard => ToolSafety::Dangerous,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "read_file" => Some(Self::ReadFile),
            "write_file" => Some(Self::WriteFile),
            "list_directory" => Some(Self::ListDirectory),
            "search_files" => Some(Self::SearchFiles),
            "delete_file" => Some(Self::DeleteFile),
            "get_system_info" => Some(Self::GetSystemInfo),
            "run_command" => Some(Self::RunCommand),
            "read_clipboard" => Some(Self::ReadClipboard),
            "write_clipboard" => Some(Self::WriteClipboard),
            _ => None,
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::ReadFile,
            Self::WriteFile,
            Self::ListDirectory,
            Self::SearchFiles,
            Self::DeleteFile,
            Self::GetSystemInfo,
            Self::RunCommand,
            Self::ReadClipboard,
            Self::WriteClipboard,
        ]
    }
}

/// Generate OpenAI-compatible tool definitions for all built-in tools
pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "read_file".to_string(),
                description: "Read the contents of a file. Returns the file content as text."
                    .to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to the file to read"
                        }
                    },
                    "required": ["path"]
                }),
            },
        },
        ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "write_file".to_string(),
                description: "Write content to a file. Creates the file if it doesn't exist, overwrites if it does.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to the file to write"
                        },
                        "content": {
                            "type": "string",
                            "description": "Content to write to the file"
                        }
                    },
                    "required": ["path", "content"]
                }),
            },
        },
        ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "list_directory".to_string(),
                description: "List the contents of a directory. Returns file and directory names with basic info.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to the directory to list"
                        }
                    },
                    "required": ["path"]
                }),
            },
        },
        ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "search_files".to_string(),
                description: "Search for files by name pattern in a directory. Returns matching file paths.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "directory": {
                            "type": "string",
                            "description": "Directory to search in"
                        },
                        "pattern": {
                            "type": "string",
                            "description": "Filename pattern to search for (supports * wildcard)"
                        }
                    },
                    "required": ["directory", "pattern"]
                }),
            },
        },
        ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "delete_file".to_string(),
                description: "Delete a file. Use with caution.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to the file to delete"
                        }
                    },
                    "required": ["path"]
                }),
            },
        },
        ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "get_system_info".to_string(),
                description: "Get system information including CPU usage, memory usage, disk space, and battery status.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
        },
        ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "run_command".to_string(),
                description: "Execute a shell command and return its output. Use with caution.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The shell command to execute"
                        }
                    },
                    "required": ["command"]
                }),
            },
        },
        ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "read_clipboard".to_string(),
                description: "Read the current contents of the system clipboard.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
        },
        ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "write_clipboard".to_string(),
                description: "Write text to the system clipboard.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "Text to write to the clipboard"
                        }
                    },
                    "required": ["content"]
                }),
            },
        },
    ]
}

/// Check if a path is within allowed directories
pub fn is_path_allowed(path: &Path, allowed_dirs: &[String]) -> bool {
    if allowed_dirs.is_empty() {
        return false;
    }

    let path = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            // If path doesn't exist yet (for writes), check parent
            if let Some(parent) = path.parent() {
                match parent.canonicalize() {
                    Ok(p) => p,
                    Err(_) => return false,
                }
            } else {
                return false;
            }
        }
    };

    for allowed in allowed_dirs {
        let allowed_path = match PathBuf::from(allowed).canonicalize() {
            Ok(p) => p,
            Err(_) => continue,
        };
        if path.starts_with(&allowed_path) {
            return true;
        }
    }
    false
}

/// Execute a tool and return the result
pub fn execute_tool(
    tool_name: &str,
    arguments: &Value,
    allowed_dirs: &[String],
) -> Result<String, String> {
    let tool =
        BuiltinTool::from_name(tool_name).ok_or_else(|| format!("Unknown tool: {}", tool_name))?;

    match tool {
        BuiltinTool::ReadFile => execute_read_file(arguments, allowed_dirs),
        BuiltinTool::WriteFile => execute_write_file(arguments, allowed_dirs),
        BuiltinTool::ListDirectory => execute_list_directory(arguments, allowed_dirs),
        BuiltinTool::SearchFiles => execute_search_files(arguments, allowed_dirs),
        BuiltinTool::DeleteFile => execute_delete_file(arguments, allowed_dirs),
        BuiltinTool::GetSystemInfo => execute_get_system_info(),
        BuiltinTool::RunCommand => execute_run_command(arguments),
        BuiltinTool::ReadClipboard => execute_read_clipboard(),
        BuiltinTool::WriteClipboard => execute_write_clipboard(arguments),
    }
}

fn execute_read_file(args: &Value, allowed_dirs: &[String]) -> Result<String, String> {
    let path_str = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'path' argument")?;
    let path = PathBuf::from(path_str);

    if !is_path_allowed(&path, allowed_dirs) {
        return Err(format!("Path '{}' is not in allowed directories", path_str));
    }

    let metadata =
        fs::metadata(&path).map_err(|e| format!("Failed to read file metadata: {}", e))?;

    if metadata.len() > MAX_FILE_READ_SIZE as u64 {
        return Err(format!(
            "File is too large ({} bytes). Maximum size is {} bytes.",
            metadata.len(),
            MAX_FILE_READ_SIZE
        ));
    }

    fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))
}

fn execute_write_file(args: &Value, allowed_dirs: &[String]) -> Result<String, String> {
    let path_str = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'path' argument")?;
    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'content' argument")?;
    let path = PathBuf::from(path_str);

    if !is_path_allowed(&path, allowed_dirs) {
        return Err(format!("Path '{}' is not in allowed directories", path_str));
    }

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directories: {}", e))?;
    }

    fs::write(&path, content).map_err(|e| format!("Failed to write file: {}", e))?;
    info!(path = %path_str, "AI tool wrote file");
    Ok(format!(
        "Successfully wrote {} bytes to {}",
        content.len(),
        path_str
    ))
}

fn execute_list_directory(args: &Value, allowed_dirs: &[String]) -> Result<String, String> {
    let path_str = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'path' argument")?;
    let path = PathBuf::from(path_str);

    if !is_path_allowed(&path, allowed_dirs) {
        return Err(format!("Path '{}' is not in allowed directories", path_str));
    }

    let entries: Vec<String> = fs::read_dir(&path)
        .map_err(|e| format!("Failed to read directory: {}", e))?
        .filter_map(|entry| {
            entry.ok().map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                let file_type = e.file_type().ok();
                let suffix = match file_type {
                    Some(ft) if ft.is_dir() => "/",
                    Some(ft) if ft.is_symlink() => "@",
                    _ => "",
                };
                format!("{}{}", name, suffix)
            })
        })
        .collect();

    Ok(entries.join("\n"))
}

fn execute_search_files(args: &Value, allowed_dirs: &[String]) -> Result<String, String> {
    let dir_str = args
        .get("directory")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'directory' argument")?;
    let pattern = args
        .get("pattern")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'pattern' argument")?;
    let dir = PathBuf::from(dir_str);

    if !is_path_allowed(&dir, allowed_dirs) {
        return Err(format!("Path '{}' is not in allowed directories", dir_str));
    }

    // Simple glob-like matching
    let pattern_regex = pattern.replace("*", ".*").replace("?", ".");
    let regex = regex::Regex::new(&format!("^{}$", pattern_regex))
        .map_err(|e| format!("Invalid pattern: {}", e))?;

    let mut matches = Vec::new();
    search_recursive(&dir, &regex, &mut matches, 5)?; // Max depth of 5

    Ok(matches.join("\n"))
}

fn search_recursive(
    dir: &Path,
    pattern: &regex::Regex,
    matches: &mut Vec<String>,
    depth: u32,
) -> Result<(), String> {
    if depth == 0 || matches.len() >= 100 {
        return Ok(());
    }

    let entries = fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();

        if pattern.is_match(&name) {
            matches.push(path.to_string_lossy().to_string());
        }

        if path.is_dir() {
            let _ = search_recursive(&path, pattern, matches, depth - 1);
        }
    }

    Ok(())
}

fn execute_delete_file(args: &Value, allowed_dirs: &[String]) -> Result<String, String> {
    let path_str = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'path' argument")?;
    let path = PathBuf::from(path_str);

    if !is_path_allowed(&path, allowed_dirs) {
        return Err(format!("Path '{}' is not in allowed directories", path_str));
    }

    if path.is_dir() {
        fs::remove_dir_all(&path).map_err(|e| format!("Failed to delete directory: {}", e))?;
    } else {
        fs::remove_file(&path).map_err(|e| format!("Failed to delete file: {}", e))?;
    }

    warn!(path = %path_str, "AI tool deleted file/directory");
    Ok(format!("Successfully deleted {}", path_str))
}

fn execute_get_system_info() -> Result<String, String> {
    use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

    let mut sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    );

    // Wait a bit for CPU stats
    std::thread::sleep(std::time::Duration::from_millis(100));
    sys.refresh_cpu_all();

    let cpu_usage: f32 =
        sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32;
    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();
    let mem_percent = (used_mem as f64 / total_mem as f64) * 100.0;

    let info = json!({
        "cpu_usage_percent": format!("{:.1}", cpu_usage),
        "memory_used_gb": format!("{:.2}", used_mem as f64 / 1_073_741_824.0),
        "memory_total_gb": format!("{:.2}", total_mem as f64 / 1_073_741_824.0),
        "memory_usage_percent": format!("{:.1}", mem_percent),
    });

    Ok(serde_json::to_string_pretty(&info).unwrap_or_default())
}

fn execute_run_command(args: &Value) -> Result<String, String> {
    let command = args
        .get("command")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'command' argument")?;

    warn!(command = %command, "AI tool executing shell command");

    let output = Command::new("bash")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        Ok(stdout.to_string())
    } else {
        Err(format!(
            "Command failed with exit code {:?}\nstdout: {}\nstderr: {}",
            output.status.code(),
            stdout,
            stderr
        ))
    }
}

fn execute_read_clipboard() -> Result<String, String> {
    // Use xclip or xsel for X11
    let output = Command::new("xclip")
        .args(["-selection", "clipboard", "-o"])
        .output()
        .or_else(|_| {
            Command::new("xsel")
                .args(["--clipboard", "--output"])
                .output()
        })
        .or_else(|_| {
            // Try wl-paste for Wayland
            Command::new("wl-paste").output()
        })
        .map_err(|e| format!("Failed to read clipboard: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err("Failed to read clipboard".to_string())
    }
}

fn execute_write_clipboard(args: &Value) -> Result<String, String> {
    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'content' argument")?;

    // Try xclip first, then xsel, then wl-copy for Wayland
    let result = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(content.as_bytes())?;
            }
            child.wait()
        })
        .or_else(|_| {
            Command::new("xsel")
                .args(["--clipboard", "--input"])
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    if let Some(stdin) = child.stdin.as_mut() {
                        stdin.write_all(content.as_bytes())?;
                    }
                    child.wait()
                })
        })
        .or_else(|_| {
            Command::new("wl-copy")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    if let Some(stdin) = child.stdin.as_mut() {
                        stdin.write_all(content.as_bytes())?;
                    }
                    child.wait()
                })
        });

    match result {
        Ok(status) if status.success() => {
            info!("AI tool wrote to clipboard");
            Ok(format!(
                "Successfully copied {} bytes to clipboard",
                content.len()
            ))
        }
        _ => Err("Failed to write to clipboard".to_string()),
    }
}

/// Models known to support function calling
static FUNCTION_CALLING_MODELS: once_cell::sync::Lazy<HashSet<&'static str>> =
    once_cell::sync::Lazy::new(|| {
        let mut s = HashSet::new();
        // OpenAI
        s.insert("openai/gpt-4o");
        s.insert("openai/gpt-4o-mini");
        s.insert("openai/gpt-4-turbo");
        s.insert("openai/gpt-4");
        s.insert("openai/gpt-4.1");
        s.insert("openai/gpt-4.1-mini");
        s.insert("openai/gpt-3.5-turbo");
        // Anthropic (via OpenRouter)
        s.insert("anthropic/claude-3-opus");
        s.insert("anthropic/claude-3-sonnet");
        s.insert("anthropic/claude-3-haiku");
        s.insert("anthropic/claude-3.7-sonnet");
        s.insert("anthropic/claude-sonnet-4");
        s.insert("anthropic/claude-opus-4");
        // Google
        s.insert("google/gemini-2.5-pro");
        s.insert("google/gemini-2.5-flash");
        s.insert("google/gemini-2.0-flash-001");
        // Mistral
        s.insert("mistralai/mistral-large");
        s.insert("mistralai/mistral-medium-3");
        s.insert("mistralai/mistral-small");
        s
    });

/// Check if a model supports function calling
pub fn model_supports_tools(model_id: &str) -> bool {
    FUNCTION_CALLING_MODELS.contains(model_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_allowed() {
        let allowed = vec!["/tmp".to_string(), "/home/test".to_string()];

        // These assertions depend on having /tmp exist
        // assert!(is_path_allowed(Path::new("/tmp/foo.txt"), &allowed));
        assert!(!is_path_allowed(Path::new("/etc/passwd"), &allowed));
    }

    #[test]
    fn test_tool_safety() {
        assert_eq!(BuiltinTool::ReadFile.safety(), ToolSafety::Safe);
        assert_eq!(BuiltinTool::WriteFile.safety(), ToolSafety::Dangerous);
        assert_eq!(BuiltinTool::RunCommand.safety(), ToolSafety::Dangerous);
    }

    #[test]
    fn test_tool_lookup() {
        assert_eq!(
            BuiltinTool::from_name("read_file"),
            Some(BuiltinTool::ReadFile)
        );
        assert_eq!(BuiltinTool::from_name("invalid"), None);
    }
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Extended tool info for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub safety: ToolSafety,
    pub definition: ToolDefinition,
}

/// Get all available AI tool definitions for the frontend
#[tauri::command]
pub fn get_ai_tool_definitions() -> Vec<ToolInfo> {
    get_tool_definitions()
        .into_iter()
        .map(|def| {
            let tool = BuiltinTool::from_name(&def.function.name);
            ToolInfo {
                name: def.function.name.clone(),
                description: def.function.description.clone(),
                safety: tool.map(|t| t.safety()).unwrap_or(ToolSafety::Dangerous),
                definition: def,
            }
        })
        .collect()
}

/// Check if a model supports function calling
#[tauri::command]
pub fn check_model_supports_tools(model_id: String) -> bool {
    model_supports_tools(&model_id)
}

/// Execute an AI tool with the given arguments
/// Returns a ToolResult with success status and output/error
#[tauri::command]
pub fn execute_ai_tool(
    tool_name: String,
    arguments: Value,
    allowed_directories: Vec<String>,
) -> ToolResult {
    let tool = BuiltinTool::from_name(&tool_name);
    let tool_call_id = format!("tool_{}", chrono::Utc::now().timestamp_millis());

    match execute_tool(&tool_name, &arguments, &allowed_directories) {
        Ok(output) => {
            info!(
                tool = %tool_name,
                safety = ?tool.map(|t| t.safety()),
                "AI tool executed successfully"
            );
            ToolResult {
                tool_call_id,
                success: true,
                output,
                error: None,
            }
        }
        Err(e) => {
            warn!(
                tool = %tool_name,
                error = %e,
                "AI tool execution failed"
            );
            ToolResult {
                tool_call_id,
                success: false,
                output: String::new(),
                error: Some(e),
            }
        }
    }
}
