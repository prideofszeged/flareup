use crate::ai::AiUsageManager;
use rusqlite::{params, Result as RusqliteResult};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

pub const AI_COMMANDS_SCHEMA: &str = r#"CREATE TABLE IF NOT EXISTS ai_commands (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    icon TEXT,
    prompt_template TEXT NOT NULL,
    model TEXT,
    creativity TEXT,
    output_action TEXT DEFAULT 'quick_ai',
    hotkey TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
)"#;

/// Output action for AI commands
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub enum OutputAction {
    #[default]
    QuickAi,
    OpenChat,
    CopyToClipboard,
    PasteInPlace,
}

/// An AI Command - a saved prompt template with placeholders
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AiCommand {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub prompt_template: String,
    pub model: Option<String>,
    pub creativity: Option<String>,
    pub output_action: OutputAction,
    pub hotkey: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl AiCommand {
    #[allow(dead_code)]
    fn from_row(row: &rusqlite::Row) -> RusqliteResult<Self> {
        let output_action_str: Option<String> = row.get(5)?;
        let output_action = match output_action_str.as_deref() {
            Some("open_chat") => OutputAction::OpenChat,
            Some("copy") => OutputAction::CopyToClipboard,
            Some("paste") => OutputAction::PasteInPlace,
            _ => OutputAction::QuickAi,
        };

        Ok(AiCommand {
            id: row.get(0)?,
            name: row.get(1)?,
            icon: row.get(2)?,
            prompt_template: row.get(3)?,
            model: row.get(4)?,
            creativity: row.get(6)?,
            output_action,
            hotkey: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    }
}

/// Create a new AI command
#[tauri::command]
pub async fn create_ai_command(
    app_handle: AppHandle,
    name: String,
    prompt_template: String,
    icon: Option<String>,
    model: Option<String>,
    creativity: Option<String>,
    output_action: Option<String>,
    hotkey: Option<String>,
) -> Result<AiCommand, String> {
    let manager = app_handle.state::<AiUsageManager>();
    let id = Uuid::new_v4().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis() as i64;

    let output_action_str = output_action
        .clone()
        .unwrap_or_else(|| "quick_ai".to_string());

    manager
        .execute_command(
            "INSERT INTO ai_commands (id, name, icon, prompt_template, model, output_action, creativity, hotkey, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![id, name, icon, prompt_template, model, output_action_str, creativity, hotkey, now, now],
        )
        .map_err(|e| e.to_string())?;

    let output_action_enum = match output_action.as_deref() {
        Some("open_chat") => OutputAction::OpenChat,
        Some("copy") => OutputAction::CopyToClipboard,
        Some("paste") => OutputAction::PasteInPlace,
        _ => OutputAction::QuickAi,
    };

    Ok(AiCommand {
        id,
        name,
        icon,
        prompt_template,
        model,
        creativity,
        output_action: output_action_enum,
        hotkey,
        created_at: now,
        updated_at: now,
    })
}

/// List all AI commands
#[tauri::command]
pub async fn list_ai_commands(app_handle: AppHandle) -> Result<Vec<AiCommand>, String> {
    let manager = app_handle.state::<AiUsageManager>();

    manager.query_ai_commands().map_err(|e| e.to_string())
}

/// Get a single AI command by ID
#[tauri::command]
pub async fn get_ai_command(
    app_handle: AppHandle,
    id: String,
) -> Result<Option<AiCommand>, String> {
    let manager = app_handle.state::<AiUsageManager>();

    manager.get_ai_command_by_id(&id).map_err(|e| e.to_string())
}

/// Update an AI command
#[tauri::command]
pub async fn update_ai_command(
    app_handle: AppHandle,
    id: String,
    name: Option<String>,
    prompt_template: Option<String>,
    icon: Option<String>,
    model: Option<String>,
    creativity: Option<String>,
    output_action: Option<String>,
    hotkey: Option<String>,
) -> Result<(), String> {
    let manager = app_handle.state::<AiUsageManager>();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis() as i64;

    // Get existing command
    let existing = manager
        .get_ai_command_by_id(&id)
        .map_err(|e| e.to_string())?;

    if let Some(existing) = existing {
        let new_name = name.unwrap_or(existing.name);
        let new_prompt = prompt_template.unwrap_or(existing.prompt_template);
        let new_icon = icon.or(existing.icon);
        let new_model = model.or(existing.model);
        let new_creativity = creativity.or(existing.creativity);
        let new_output_action = output_action.unwrap_or_else(|| {
            match existing.output_action {
                OutputAction::QuickAi => "quick_ai",
                OutputAction::OpenChat => "open_chat",
                OutputAction::CopyToClipboard => "copy",
                OutputAction::PasteInPlace => "paste",
            }
            .to_string()
        });
        let new_hotkey = hotkey.or(existing.hotkey);

        manager
            .execute_command(
                "UPDATE ai_commands SET name = ?1, prompt_template = ?2, icon = ?3, model = ?4, creativity = ?5, output_action = ?6, hotkey = ?7, updated_at = ?8 WHERE id = ?9",
                params![new_name, new_prompt, new_icon, new_model, new_creativity, new_output_action, new_hotkey, now, id],
            )
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Delete an AI command
#[tauri::command]
pub async fn delete_ai_command(app_handle: AppHandle, id: String) -> Result<(), String> {
    let manager = app_handle.state::<AiUsageManager>();

    manager
        .execute_command("DELETE FROM ai_commands WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Substitute placeholders in a prompt template
/// Supported placeholders:
/// - {selection} - Currently selected text
/// - {clipboard} - Current clipboard content
/// - {input} - User-provided input (passed as argument)
/// - {browser_text} - Text from browser (if available)
#[tauri::command]
pub async fn substitute_placeholders(
    prompt_template: String,
    selection: Option<String>,
    clipboard: Option<String>,
    input: Option<String>,
    browser_text: Option<String>,
) -> Result<String, String> {
    let mut result = prompt_template;

    // Substitute placeholders
    result = result.replace("{selection}", &selection.unwrap_or_default());
    result = result.replace("{clipboard}", &clipboard.unwrap_or_default());
    result = result.replace("{input}", &input.unwrap_or_default());
    result = result.replace("{browser_text}", &browser_text.unwrap_or_default());

    Ok(result)
}

/// Get available placeholder names for UI hints
#[tauri::command]
pub fn get_available_placeholders() -> Vec<PlaceholderInfo> {
    vec![
        PlaceholderInfo {
            name: "{selection}".to_string(),
            description: "Currently selected text from any app".to_string(),
        },
        PlaceholderInfo {
            name: "{clipboard}".to_string(),
            description: "Current clipboard content".to_string(),
        },
        PlaceholderInfo {
            name: "{input}".to_string(),
            description: "Input you type when running the command".to_string(),
        },
        PlaceholderInfo {
            name: "{browser_text}".to_string(),
            description: "Text from the browser extension (if connected)".to_string(),
        },
    ]
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaceholderInfo {
    pub name: String,
    pub description: String,
}
