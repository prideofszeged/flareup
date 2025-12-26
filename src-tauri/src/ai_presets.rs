use crate::ai::AiUsageManager;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

pub const AI_PRESETS_SCHEMA: &str = r#"CREATE TABLE IF NOT EXISTS ai_presets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    icon TEXT,
    model TEXT,
    temperature REAL,
    system_prompt TEXT,
    web_search INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
)"#;

/// An AI Chat Preset - saved configuration for different use cases
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AiPreset {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub model: Option<String>,
    pub temperature: Option<f64>,
    pub system_prompt: Option<String>,
    pub web_search: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Create a new AI preset
#[tauri::command]
pub async fn create_ai_preset(
    app_handle: AppHandle,
    name: String,
    icon: Option<String>,
    model: Option<String>,
    temperature: Option<f64>,
    system_prompt: Option<String>,
    web_search: Option<bool>,
) -> Result<AiPreset, String> {
    let manager = app_handle.state::<AiUsageManager>();
    let id = Uuid::new_v4().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis() as i64;

    let web_search_val = web_search.unwrap_or(false);

    manager
        .execute_command(
            "INSERT INTO ai_presets (id, name, icon, model, temperature, system_prompt, web_search, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![id, name, icon, model, temperature, system_prompt, web_search_val, now, now],
        )
        .map_err(|e| e.to_string())?;

    Ok(AiPreset {
        id,
        name,
        icon,
        model,
        temperature,
        system_prompt,
        web_search: web_search_val,
        created_at: now,
        updated_at: now,
    })
}

/// List all AI presets
#[tauri::command]
pub async fn list_ai_presets(app_handle: AppHandle) -> Result<Vec<AiPreset>, String> {
    let manager = app_handle.state::<AiUsageManager>();
    manager.query_ai_presets().map_err(|e| e.to_string())
}

/// Get a single AI preset by ID
#[tauri::command]
pub async fn get_ai_preset(app_handle: AppHandle, id: String) -> Result<Option<AiPreset>, String> {
    let manager = app_handle.state::<AiUsageManager>();
    manager.get_ai_preset_by_id(&id).map_err(|e| e.to_string())
}

/// Update an AI preset
#[tauri::command]
pub async fn update_ai_preset(
    app_handle: AppHandle,
    id: String,
    name: Option<String>,
    icon: Option<String>,
    model: Option<String>,
    temperature: Option<f64>,
    system_prompt: Option<String>,
    web_search: Option<bool>,
) -> Result<(), String> {
    let manager = app_handle.state::<AiUsageManager>();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis() as i64;

    // Get existing preset
    let existing = manager
        .get_ai_preset_by_id(&id)
        .map_err(|e| e.to_string())?;

    if let Some(existing) = existing {
        let new_name = name.unwrap_or(existing.name);
        let new_icon = icon.or(existing.icon);
        let new_model = model.or(existing.model);
        let new_temperature = temperature.or(existing.temperature);
        let new_system_prompt = system_prompt.or(existing.system_prompt);
        let new_web_search = web_search.unwrap_or(existing.web_search);

        manager
            .execute_command(
                "UPDATE ai_presets SET name = ?1, icon = ?2, model = ?3, temperature = ?4, system_prompt = ?5, web_search = ?6, updated_at = ?7 WHERE id = ?8",
                params![new_name, new_icon, new_model, new_temperature, new_system_prompt, new_web_search, now, id],
            )
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Delete an AI preset
#[tauri::command]
pub async fn delete_ai_preset(app_handle: AppHandle, id: String) -> Result<(), String> {
    let manager = app_handle.state::<AiUsageManager>();

    manager
        .execute_command("DELETE FROM ai_presets WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}
