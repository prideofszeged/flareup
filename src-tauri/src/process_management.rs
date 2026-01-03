//! Process management module for listing, searching, and killing processes
//! Also provides port-to-process lookup functionality

use serde::{Deserialize, Serialize};
use std::process::Command;

/// Information about a running process
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub command: String,
    pub cpu_percent: f32,
    pub memory_percent: f32,
    pub memory_mb: f32,
    pub user: String,
}

/// Information about an open port
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortInfo {
    pub port: u16,
    pub protocol: String,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
    pub local_address: String,
    pub state: String,
}

/// Signal type for killing processes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum KillSignal {
    /// SIGTERM - graceful termination request
    Soft,
    /// SIGKILL - forceful immediate termination
    Hard,
}

/// List all running processes
#[tauri::command]
pub async fn process_list() -> Result<Vec<ProcessInfo>, String> {
    tracing::info!("Listing all processes");

    // Use ps to get process information
    // Format: PID, USER, %CPU, %MEM, RSS (KB), COMM, ARGS
    let output = Command::new("ps")
        .args([
            "-eo",
            "pid,user,%cpu,%mem,rss,comm,args",
            "--no-headers",
            "--sort=-%cpu",
        ])
        .output()
        .map_err(|e| format!("Failed to execute ps: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ps command failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut processes = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            let pid = parts[0].parse::<u32>().unwrap_or(0);
            let user = parts[1].to_string();
            let cpu_percent = parts[2].parse::<f32>().unwrap_or(0.0);
            let memory_percent = parts[3].parse::<f32>().unwrap_or(0.0);
            let rss_kb = parts[4].parse::<f32>().unwrap_or(0.0);
            let name = parts[5].to_string();
            let command = parts[6..].join(" ");

            processes.push(ProcessInfo {
                pid,
                name,
                command,
                cpu_percent,
                memory_percent,
                memory_mb: rss_kb / 1024.0,
                user,
            });
        }
    }

    tracing::info!("Found {} processes", processes.len());
    Ok(processes)
}

/// Information about a GUI window/application
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowInfo {
    pub pid: u32,
    pub window_id: String,
    pub title: String,
    pub class_name: String,
}

/// List all open GUI windows/applications (for Quick Kill feature)
#[tauri::command]
pub async fn list_windows() -> Result<Vec<WindowInfo>, String> {
    tracing::info!("Listing GUI windows");

    // Try wmctrl first (most common), fall back to xdotool
    if command_exists("wmctrl") {
        return list_windows_wmctrl().await;
    }

    if command_exists("xdotool") {
        return list_windows_xdotool().await;
    }

    Err("Neither 'wmctrl' nor 'xdotool' found. Install wmctrl for best results.".to_string())
}

/// List windows using wmctrl
async fn list_windows_wmctrl() -> Result<Vec<WindowInfo>, String> {
    // wmctrl -l -p gives: window_id desktop pid host title
    let output = Command::new("wmctrl")
        .args(["-l", "-p", "-x"]) // -x adds class name
        .output()
        .map_err(|e| format!("Failed to execute wmctrl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("wmctrl failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut windows = Vec::new();

    for line in stdout.lines() {
        // Format: 0x04000007  0 12345  class.ClassName  hostname Window Title
        let parts: Vec<&str> = line.splitn(5, |c: char| c.is_whitespace()).collect();
        if parts.len() >= 5 {
            let window_id = parts[0].to_string();
            // Skip the desktop number (parts[1])
            let pid = parts[2].trim().parse::<u32>().unwrap_or(0);
            let class_name = parts[3].trim().to_string();
            // The rest after the 4th whitespace-separated field is the title
            let title = line
                .splitn(5, char::is_whitespace)
                .nth(4)
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            // Skip windows without PIDs or with empty titles
            if pid > 0 && !title.is_empty() {
                windows.push(WindowInfo {
                    pid,
                    window_id,
                    title,
                    class_name,
                });
            }
        }
    }

    // Deduplicate by PID (keep first window of each app)
    let mut seen_pids = std::collections::HashSet::new();
    windows.retain(|w| seen_pids.insert(w.pid));

    tracing::info!("Found {} GUI windows", windows.len());
    Ok(windows)
}

/// List windows using xdotool (fallback)
async fn list_windows_xdotool() -> Result<Vec<WindowInfo>, String> {
    // Use --class "" to search all visible windows (better coverage than --name "")
    let output = Command::new("xdotool")
        .args(["search", "--onlyvisible", "--class", ""])
        .output()
        .map_err(|e| format!("Failed to execute xdotool: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut windows = Vec::new();

    for window_id in stdout.lines() {
        let window_id = window_id.trim();
        if window_id.is_empty() {
            continue;
        }

        // Get window name
        let name_output = Command::new("xdotool")
            .args(["getwindowname", window_id])
            .output()
            .ok();

        let title = name_output
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        // Get PID
        let pid_output = Command::new("xdotool")
            .args(["getwindowpid", window_id])
            .output()
            .ok();

        let pid = pid_output
            .and_then(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .parse::<u32>()
                    .ok()
            })
            .unwrap_or(0);

        if pid > 0 && !title.is_empty() {
            windows.push(WindowInfo {
                pid,
                window_id: window_id.to_string(),
                title,
                class_name: String::new(), // xdotool doesn't easily give class
            });
        }
    }

    // Deduplicate by PID
    let mut seen_pids = std::collections::HashSet::new();
    windows.retain(|w| seen_pids.insert(w.pid));

    tracing::info!("Found {} GUI windows via xdotool", windows.len());
    Ok(windows)
}

/// Search processes by name or command
#[tauri::command]
pub async fn process_search(query: String) -> Result<Vec<ProcessInfo>, String> {
    tracing::info!("Searching processes with query: {}", query);

    let all_processes = process_list().await?;
    let query_lower = query.to_lowercase();

    let filtered: Vec<ProcessInfo> = all_processes
        .into_iter()
        .filter(|p| {
            p.name.to_lowercase().contains(&query_lower)
                || p.command.to_lowercase().contains(&query_lower)
                || p.pid.to_string().contains(&query)
        })
        .collect();

    tracing::info!("Found {} matching processes", filtered.len());
    Ok(filtered)
}

/// Kill a process by PID
#[tauri::command]
pub async fn process_kill(pid: u32, signal: KillSignal) -> Result<(), String> {
    let signal_name = match signal {
        KillSignal::Soft => "SIGTERM",
        KillSignal::Hard => "SIGKILL",
    };

    tracing::info!("Killing process {} with {}", pid, signal_name);

    let signal_arg = match signal {
        KillSignal::Soft => "-15", // SIGTERM
        KillSignal::Hard => "-9",  // SIGKILL
    };

    let output = Command::new("kill")
        .args([signal_arg, &pid.to_string()])
        .output()
        .map_err(|e| format!("Failed to execute kill: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Check for common errors
        if stderr.contains("No such process") {
            return Err(format!("Process {} does not exist", pid));
        } else if stderr.contains("Operation not permitted") {
            return Err(format!(
                "Permission denied: cannot kill process {}. Try running as root.",
                pid
            ));
        }
        return Err(format!("Failed to kill process {}: {}", pid, stderr));
    }

    tracing::info!("Successfully sent {} to process {}", signal_name, pid);
    Ok(())
}

/// List all open ports with their processes
#[tauri::command]
pub async fn port_list_open() -> Result<Vec<PortInfo>, String> {
    tracing::info!("Listing open ports");

    // Try ss first (modern), fall back to netstat
    let output = if command_exists("ss") {
        Command::new("ss")
            .args(["-tlnp"]) // TCP listening, numeric, show process
            .output()
            .map_err(|e| format!("Failed to execute ss: {}", e))?
    } else if command_exists("netstat") {
        Command::new("netstat")
            .args(["-tlnp"]) // TCP listening, numeric, show process
            .output()
            .map_err(|e| format!("Failed to execute netstat: {}", e))?
    } else {
        return Err("Neither 'ss' nor 'netstat' found. Install iproute2 or net-tools.".to_string());
    };

    if !output.status.success() {
        // ss/netstat may fail without root for some info, but partial output is still useful
        tracing::warn!("Port listing command returned non-zero exit code, parsing partial output");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut ports = Vec::new();

    for line in stdout.lines().skip(1) {
        // Skip header
        if let Some(port_info) = parse_ss_line(line) {
            ports.push(port_info);
        }
    }

    // Also get UDP ports
    let udp_output = if command_exists("ss") {
        Command::new("ss")
            .args(["-ulnp"]) // UDP listening, numeric, show process
            .output()
            .ok()
    } else {
        Command::new("netstat").args(["-ulnp"]).output().ok()
    };

    if let Some(output) = udp_output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(1) {
            if let Some(mut port_info) = parse_ss_line(line) {
                port_info.protocol = "UDP".to_string();
                ports.push(port_info);
            }
        }
    }

    // Sort by port number
    ports.sort_by_key(|p| p.port);

    tracing::info!("Found {} open ports", ports.len());
    Ok(ports)
}

/// Get process information for a specific port
#[tauri::command]
pub async fn port_get_process(port: u16) -> Result<Option<PortInfo>, String> {
    tracing::info!("Looking up process on port {}", port);

    let all_ports = port_list_open().await?;
    let found = all_ports.into_iter().find(|p| p.port == port);

    if found.is_some() {
        tracing::info!("Found process on port {}", port);
    } else {
        tracing::info!("No process found on port {}", port);
    }

    Ok(found)
}

/// Kill the process using a specific port
#[tauri::command]
pub async fn port_kill_process(port: u16, signal: KillSignal) -> Result<(), String> {
    tracing::info!("Killing process on port {}", port);

    let port_info = port_get_process(port).await?;

    match port_info {
        Some(info) => match info.pid {
            Some(pid) => {
                process_kill(pid, signal).await?;
                tracing::info!("Successfully killed process {} on port {}", pid, port);
                Ok(())
            }
            None => Err(format!(
                "Cannot determine PID for port {}. Try running as root.",
                port
            )),
        },
        None => Err(format!("No process found listening on port {}", port)),
    }
}

/// Check if a command exists in PATH
fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Parse a line from ss output
/// Actual ss -tlnp format: State Recv-Q Send-Q LocalAddress:Port PeerAddress:Port Process
/// Example: "LISTEN 0 100 0.0.0.0:4000 0.0.0.0:*"
/// Example with process: "LISTEN 0 100 127.0.0.1:25003 0.0.0.0:* users:(("nxrunner.bin",pid=1604895,fd=14))"
fn parse_ss_line(line: &str) -> Option<PortInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    // Need at least: State, Recv-Q, Send-Q, LocalAddr, PeerAddr
    if parts.len() < 5 {
        return None;
    }

    // For ss output format: State Recv-Q Send-Q LocalAddr:Port PeerAddr:Port [Process]
    let state = parts[0].to_string();
    let local_addr = parts[3]; // Local address:port is the 4th column (0-indexed: 3)

    // Parse port from address (e.g., "0.0.0.0:8080" or "[::]:22" or "192.168.1.1:80")
    let port = if local_addr.contains("]:") {
        // IPv6 format: [::]:port
        local_addr.split("]:").last()?.parse::<u16>().ok()?
    } else if local_addr.contains(':') {
        // IPv4 format: addr:port
        local_addr.split(':').last()?.parse::<u16>().ok()?
    } else {
        return None;
    };

    // Try to extract PID and process name from users column (if present)
    // Format: users:(("process_name",pid=1234,fd=20))
    let mut pid: Option<u32> = None;
    let mut process_name: Option<String> = None;

    // The rest of the line after peer address may contain process info
    let full_line = line;
    if full_line.contains("pid=") {
        // Extract PID
        if let Some(pid_start) = full_line.find("pid=") {
            let pid_str = &full_line[pid_start + 4..];
            pid = pid_str
                .split(|c: char| !c.is_ascii_digit())
                .next()
                .and_then(|s| s.parse::<u32>().ok());
        }
        // Extract process name - appears in (("name",pid=...))
        if let Some(start) = full_line.find("((\"") {
            if let Some(end) = full_line[start + 3..].find('"') {
                process_name = Some(full_line[start + 3..start + 3 + end].to_string());
            }
        }
    }

    Some(PortInfo {
        port,
        protocol: "TCP".to_string(),
        pid,
        process_name,
        local_address: local_addr.to_string(),
        state,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ss_line_basic() {
        // Actual ss -tlnp output format
        let line =
            "LISTEN 0 100 127.0.0.1:25003 0.0.0.0:* users:((\"nxrunner.bin\",pid=1604895,fd=14))";
        let result = parse_ss_line(line);
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.port, 25003);
        assert_eq!(info.pid, Some(1604895));
        assert_eq!(info.process_name, Some("nxrunner.bin".to_string()));
        assert_eq!(info.state, "LISTEN");
    }

    #[test]
    fn test_parse_ss_line_ipv6() {
        let line = "LISTEN 0 128 [::]:22 [::]:* users:((\"sshd\",pid=567,fd=4))";
        let result = parse_ss_line(line);
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.port, 22);
        assert_eq!(info.pid, Some(567));
        assert_eq!(info.process_name, Some("sshd".to_string()));
    }

    #[test]
    fn test_parse_ss_line_no_process() {
        // Port without process info (no root permissions to see process)
        let line = "LISTEN 0 4096 0.0.0.0:22 0.0.0.0:*";
        let result = parse_ss_line(line);
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.port, 22);
        assert!(info.pid.is_none());
        assert!(info.process_name.is_none());
    }

    #[test]
    fn test_parse_ss_line_with_ip() {
        let line = "LISTEN 0 32 192.168.122.1:53 0.0.0.0:*";
        let result = parse_ss_line(line);
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.port, 53);
        assert_eq!(info.local_address, "192.168.122.1:53");
    }
}
