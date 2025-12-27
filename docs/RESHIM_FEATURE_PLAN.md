# Reshim Feature Implementation Plan

> **Status**: Draft - Under Consideration

## Goal

Add an active "Reshim" feature to the Extensions Settings panel that attempts to fix compatibility issues by automatically installing Linux shims for detected macOS dependencies.

## Proposed Changes

### Backend (Rust)

#### `shim_registry.rs`

Add a new function that performs a full reshim analysis:

```rust
pub fn analyze_extension_for_reshim(extension_path: &str) -> Result<ReshimAnalysis, String>
```

Returns:

- Tools that can be shimmed
- Tools already shimmed
- Tools that cannot be shimmed (need manual install)

#### `lib.rs`

Add new Tauri commands:

```rust
#[tauri::command]
fn shim_analyze_extension(extension_path: String) -> Result<ReshimAnalysis, String>

#[tauri::command]
fn shim_apply_reshim(tool_names: Vec<String>) -> Result<ReshimResult, String>
```

### Frontend (Svelte)

#### `ExtensionsSettings.svelte`

Add a "Try to Fix" button in the compatibility warnings panel:

1. When clicked, calls `shim_analyze_extension` with the extension path
2. Shows a modal/dialog with:
   - Tools that can be auto-shimmed
   - Tools that need manual installation
   - "Apply Shims" button
3. On apply, calls `shim_apply_reshim` and shows success/failure HUD

## UI Design

When compatibility warnings are shown:

```
┌─────────────────────────────────────────────────┐
│ ⚠️ Potential compatibility issues detected      │
│                                                 │
│ • osascript: AppleScript detected               │
│ • pbcopy: Clipboard command detected            │
│                                                 │
│ [🔧 Try to Fix]  [ℹ️ Learn More]                │
└─────────────────────────────────────────────────┘
```

Clicking "Try to Fix" shows:

```
┌─────────────────────────────────────────────────┐
│ Compatibility Analysis                          │
│─────────────────────────────────────────────────│
│ ✅ Can be shimmed (2):                          │
│    • pbcopy → xclip wrapper                     │
│    • pbpaste → xclip wrapper                    │
│                                                 │
│ ⚠️ Needs manual install (1):                    │
│    • speedtest → Install speedtest-cli          │
│                                                 │
│ ❌ Cannot be shimmed (1):                       │
│    • Custom Mach-O binary                       │
│                                                 │
│ [Apply Shims]  [Cancel]                         │
└─────────────────────────────────────────────────┘
```

## Data Structures

```rust
#[derive(Serialize, Deserialize)]
pub struct ReshimAnalysis {
    pub can_shim: Vec<ToolMapping>,
    pub already_shimmed: Vec<String>,
    pub needs_install: Vec<ToolMapping>,
    pub cannot_shim: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ReshimResult {
    pub shimmed: Vec<String>,
    pub failed: Vec<(String, String)>, // (tool_name, error)
}
```

## Prerequisites

This builds on the existing shim registry system in `src-tauri/src/shim_registry.rs` which already has:

- Tool registry with macOS → Linux mappings
- Wrapper script generation
- Distro detection
- Tool installation checking

## Verification Plan

1. Install an extension with known compatibility warnings
2. Click "Try to Fix" and verify analysis is shown
3. Apply shims and verify wrapper scripts are created in `~/.local/share/flareup/shims/`
4. Verify HUD shows success message
5. Re-check compatibility - warnings should be reduced
