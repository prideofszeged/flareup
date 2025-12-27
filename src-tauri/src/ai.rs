use crate::error::AppError;
use crate::store::{Storable, Store};
use futures_util::StreamExt;
use once_cell::sync::Lazy;
use rusqlite::{params, Result as RusqliteResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager, State};

const AI_KEYRING_SERVICE: &str = "dev.byteatatime.flare.ai";
const AI_KEYRING_USERNAME: &str = "openrouter_api_key";
const AI_USAGE_SCHEMA: &str = "CREATE TABLE IF NOT EXISTS ai_generations (
    id TEXT PRIMARY KEY,
    created INTEGER NOT NULL,
    model TEXT NOT NULL,
    tokens_prompt INTEGER NOT NULL,
    tokens_completion INTEGER NOT NULL,
    native_tokens_prompt INTEGER NOT NULL,
    native_tokens_completion INTEGER NOT NULL,
    total_cost REAL NOT NULL
)";

const AI_CONVERSATIONS_SCHEMA: &str = "CREATE TABLE IF NOT EXISTS ai_conversations (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    model TEXT,
    messages TEXT NOT NULL
)";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AskOptions {
    pub model: Option<String>,
    pub creativity: Option<String>,
    #[serde(default)]
    pub enable_tools: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StreamChunk {
    request_id: String,
    text: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StreamEnd {
    request_id: String,
    full_text: String,
}

/// Event emitted when AI requests a tool call
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallRequest {
    pub request_id: String,
    pub tool_call_id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub safety: String, // "safe" or "dangerous"
}

/// Event emitted when a tool execution completes
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallResult {
    pub request_id: String,
    pub tool_call_id: String,
    pub tool_name: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub model: Option<String>,
    pub messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenerationData {
    pub id: String,
    pub created: i64,
    pub model: String,
    #[serde(default)]
    pub tokens_prompt: i64,
    #[serde(default)]
    pub tokens_completion: i64,
    #[serde(default)]
    pub native_tokens_prompt: i64,
    #[serde(default)]
    pub native_tokens_completion: i64,
    #[serde(default)]
    pub total_cost: f64,
}

impl Storable for GenerationData {
    fn from_row(row: &rusqlite::Row) -> RusqliteResult<Self> {
        Ok(GenerationData {
            id: row.get(0)?,
            created: row.get(1)?,
            model: row.get(2)?,
            tokens_prompt: row.get(3)?,
            tokens_completion: row.get(4)?,
            native_tokens_prompt: row.get(5)?,
            native_tokens_completion: row.get(6)?,
            total_cost: row.get(7)?,
        })
    }
}

static DEFAULT_AI_MODELS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // OpenAI
    m.insert("OpenAI_GPT4.1", "openai/gpt-4.1");
    m.insert("OpenAI_GPT4.1-mini", "openai/gpt-4.1-mini");
    m.insert("OpenAI_GPT4.1-nano", "openai/gpt-4.1-nano");
    m.insert("OpenAI_GPT4", "openai/gpt-4");
    m.insert("OpenAI_GPT4-turbo", "openai/gpt-4-turbo");
    m.insert("OpenAI_GPT4o", "openai/gpt-4o");
    m.insert("OpenAI_GPT4o-mini", "openai/gpt-4o-mini");
    m.insert("OpenAI_o3", "openai/o3");
    m.insert("OpenAI_o4-mini", "openai/o4-mini");
    m.insert("OpenAI_o1", "openai/o1");
    m.insert("OpenAI_o3-mini", "openai/o3-mini");
    // Anthropic
    m.insert("Anthropic_Claude_Haiku", "anthropic/claude-3-haiku");
    m.insert("Anthropic_Claude_Sonnet", "anthropic/claude-3-sonnet");
    m.insert("Anthropic_Claude_Sonnet_3.7", "anthropic/claude-3.7-sonnet");
    m.insert("Anthropic_Claude_Opus", "anthropic/claude-3-opus");
    m.insert("Anthropic_Claude_4_Sonnet", "anthropic/claude-sonnet-4");
    m.insert("Anthropic_Claude_4_Opus", "anthropic/claude-opus-4");
    // Perplexity
    m.insert("Perplexity_Sonar", "perplexity/sonar");
    m.insert("Perplexity_Sonar_Pro", "perplexity/sonar-pro");
    m.insert("Perplexity_Sonar_Reasoning", "perplexity/sonar-reasoning");
    m.insert(
        "Perplexity_Sonar_Reasoning_Pro",
        "perplexity/sonar-reasoning-pro",
    );
    // Meta
    m.insert("Llama4_Scout", "meta-llama/llama-4-scout");
    m.insert("Llama3.3_70B", "meta-llama/llama-3.3-70b-instruct");
    m.insert("Llama3.1_8B", "meta-llama/llama-3.1-8b-instruct");
    m.insert("Llama3.1_405B", "meta-llama/llama-3.1-405b-instruct");
    // Mistral
    m.insert("Mistral_Nemo", "mistralai/mistral-nemo");
    m.insert("Mistral_Large", "mistralai/mistral-large");
    m.insert("Mistral_Medium", "mistralai/mistral-medium-3");
    m.insert("Mistral_Small", "mistralai/mistral-small");
    m.insert("Mistral_Codestral", "mistralai/codestral-2501");
    // DeepSeek
    m.insert(
        "DeepSeek_R1_Distill_Llama_3.3_70B",
        "deepseek/deepseek-r1-distill-llama-70b",
    );
    m.insert("DeepSeek_R1", "deepseek/deepseek-r1");
    m.insert("DeepSeek_V3", "deepseek/deepseek-chat");
    // Google
    m.insert("Google_Gemini_2.5_Pro", "google/gemini-2.5-pro");
    m.insert("Google_Gemini_2.5_Flash", "google/gemini-2.5-flash");
    m.insert("Google_Gemini_2.0_Flash", "google/gemini-2.0-flash-001");
    // xAI
    m.insert("xAI_Grok_3", "x-ai/grok-3");
    m.insert("xAI_Grok_3_Mini", "x-ai/grok-3-mini");
    m.insert("xAI_Grok_2", "x-ai/grok-2-1212");

    m
});

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AiProvider {
    OpenRouter,
    Ollama,
}

impl Default for AiProvider {
    fn default() -> Self {
        AiProvider::OpenRouter
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AiSettings {
    enabled: bool,
    #[serde(default)]
    provider: AiProvider,
    #[serde(default)]
    base_url: Option<String>,
    #[serde(default = "default_temperature")]
    temperature: f64,
    model_associations: HashMap<String, String>,
    // Tool use settings
    #[serde(default)]
    pub tools_enabled: bool,
    #[serde(default)]
    pub allowed_directories: Vec<String>,
    #[serde(default = "default_true")]
    pub auto_approve_safe_tools: bool,
    #[serde(default)]
    pub auto_approve_all_tools: bool,
}

fn default_true() -> bool {
    true
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: AiProvider::default(),
            base_url: None,
            temperature: default_temperature(),
            model_associations: HashMap::new(),
            tools_enabled: false,
            allowed_directories: Vec::new(),
            auto_approve_safe_tools: true,
            auto_approve_all_tools: false,
        }
    }
}

fn default_temperature() -> f64 {
    0.7
}

fn get_settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|_| "Failed to get app local data dir".to_string())?;

    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    }
    Ok(data_dir.join("ai_settings.json"))
}

fn read_settings(path: &Path) -> Result<AiSettings, String> {
    if !path.exists() {
        return Ok(AiSettings::default());
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    if content.trim().is_empty() {
        return Ok(AiSettings::default());
    }
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

fn write_settings(path: &Path, settings: &AiSettings) -> Result<(), String> {
    let content = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_ai_settings(app: tauri::AppHandle) -> Result<AiSettings, String> {
    let path = get_settings_path(&app)?;
    let mut user_settings = read_settings(&path)?;

    for (key, &default_value) in DEFAULT_AI_MODELS.iter() {
        let entry = user_settings
            .model_associations
            .entry(key.to_string())
            .or_insert_with(|| default_value.to_string());

        if entry.is_empty() {
            *entry = default_value.to_string();
        }
    }

    Ok(user_settings)
}

#[tauri::command]
pub fn set_ai_settings(app: tauri::AppHandle, settings: AiSettings) -> Result<(), String> {
    let path = get_settings_path(&app)?;

    let mut settings_to_save = AiSettings {
        enabled: settings.enabled,
        provider: settings.provider,
        base_url: settings.base_url,
        temperature: settings.temperature,
        model_associations: HashMap::new(),
        tools_enabled: settings.tools_enabled,
        allowed_directories: settings.allowed_directories,
        auto_approve_safe_tools: settings.auto_approve_safe_tools,
        auto_approve_all_tools: settings.auto_approve_all_tools,
    };

    for (key, value) in settings.model_associations {
        let is_different_from_default = DEFAULT_AI_MODELS
            .get(key.as_str())
            .map_or(true, |&default_val| default_val != value);

        if is_different_from_default {
            settings_to_save.model_associations.insert(key, value);
        }
    }

    write_settings(&path, &settings_to_save)
}

fn get_keyring_entry() -> Result<keyring::Entry, AppError> {
    keyring::Entry::new(AI_KEYRING_SERVICE, AI_KEYRING_USERNAME).map_err(AppError::from)
}

#[tauri::command]
pub fn set_ai_api_key(key: String) -> Result<(), String> {
    get_keyring_entry()
        .and_then(|entry| entry.set_password(&key).map_err(AppError::from))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn is_ai_api_key_set() -> Result<bool, String> {
    match get_keyring_entry().and_then(|entry| entry.get_password().map_err(AppError::from)) {
        Ok(_) => Ok(true),
        Err(AppError::Keyring(keyring::Error::NoEntry)) => Ok(false),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn clear_ai_api_key() -> Result<(), String> {
    get_keyring_entry()
        .and_then(|entry| entry.delete_credential().map_err(AppError::from))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ai_can_access(app: tauri::AppHandle) -> Result<bool, String> {
    let settings = get_ai_settings(app)?;
    if !settings.enabled {
        return Ok(false);
    }
    is_ai_api_key_set()
}

pub struct AiUsageManager {
    store: Store,
}

impl AiUsageManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self, AppError> {
        let store = Store::new(app_handle, "ai_usage.sqlite")?;
        store.init_table(AI_USAGE_SCHEMA)?;
        store.init_table(AI_CONVERSATIONS_SCHEMA)?;

        // Initialize AI commands table
        store.init_table(crate::ai_commands::AI_COMMANDS_SCHEMA)?;

        // Initialize AI presets table
        store.init_table(crate::ai_presets::AI_PRESETS_SCHEMA)?;

        // Add indices for performance
        store.execute(
            "CREATE INDEX IF NOT EXISTS idx_ai_generations_created ON ai_generations(created)",
            params![],
        )?;
        store.execute(
            "CREATE INDEX IF NOT EXISTS idx_ai_conversations_updated ON ai_conversations(updated_at)",
            params![],
        )?;

        Ok(Self { store })
    }

    pub fn log_generation(&self, data: &GenerationData) -> Result<(), AppError> {
        self.store.execute(
            "INSERT OR REPLACE INTO ai_generations (id, created, model, tokens_prompt, tokens_completion, native_tokens_prompt, native_tokens_completion, total_cost)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                data.id,
                data.created,
                data.model,
                data.tokens_prompt,
                data.tokens_completion,
                data.native_tokens_prompt,
                data.native_tokens_completion,
                data.total_cost
            ],
        )?;
        Ok(())
    }

    pub fn get_history(&self, limit: u32, offset: u32) -> Result<Vec<GenerationData>, AppError> {
        self.store.query(
            "SELECT id, created, model, tokens_prompt, tokens_completion, native_tokens_prompt, native_tokens_completion, total_cost FROM ai_generations ORDER BY created DESC LIMIT ?1 OFFSET ?2",
            params![limit, offset],
        )
    }

    /// Execute a command on the store (for AI commands table)
    pub fn execute_command<P: rusqlite::Params>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<usize, AppError> {
        self.store.execute(sql, params)
    }

    /// Query all AI commands
    pub fn query_ai_commands(&self) -> Result<Vec<crate::ai_commands::AiCommand>, AppError> {
        let conn = self.store.conn();
        let mut stmt = conn.prepare(
            "SELECT id, name, icon, prompt_template, model, output_action, creativity, hotkey, created_at, updated_at FROM ai_commands ORDER BY name ASC"
        )?;

        let commands = stmt
            .query_map([], |row| {
                let output_action_str: Option<String> = row.get(5)?;
                let output_action = match output_action_str.as_deref() {
                    Some("open_chat") => crate::ai_commands::OutputAction::OpenChat,
                    Some("copy") => crate::ai_commands::OutputAction::CopyToClipboard,
                    Some("paste") => crate::ai_commands::OutputAction::PasteInPlace,
                    _ => crate::ai_commands::OutputAction::QuickAi,
                };

                Ok(crate::ai_commands::AiCommand {
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
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(commands)
    }

    /// Get a single AI command by ID
    pub fn get_ai_command_by_id(
        &self,
        id: &str,
    ) -> Result<Option<crate::ai_commands::AiCommand>, AppError> {
        let conn = self.store.conn();
        let mut stmt = conn.prepare(
            "SELECT id, name, icon, prompt_template, model, output_action, creativity, hotkey, created_at, updated_at FROM ai_commands WHERE id = ?1"
        )?;

        let result = stmt
            .query_row(params![id], |row| {
                let output_action_str: Option<String> = row.get(5)?;
                let output_action = match output_action_str.as_deref() {
                    Some("open_chat") => crate::ai_commands::OutputAction::OpenChat,
                    Some("copy") => crate::ai_commands::OutputAction::CopyToClipboard,
                    Some("paste") => crate::ai_commands::OutputAction::PasteInPlace,
                    _ => crate::ai_commands::OutputAction::QuickAi,
                };

                Ok(crate::ai_commands::AiCommand {
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
            })
            .ok();

        Ok(result)
    }

    /// Query all AI presets
    pub fn query_ai_presets(&self) -> Result<Vec<crate::ai_presets::AiPreset>, AppError> {
        let conn = self.store.conn();
        let mut stmt = conn.prepare(
            "SELECT id, name, icon, model, temperature, system_prompt, web_search, created_at, updated_at FROM ai_presets ORDER BY name ASC"
        )?;

        let presets = stmt
            .query_map([], |row| {
                let web_search: i32 = row.get(6)?;
                Ok(crate::ai_presets::AiPreset {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    icon: row.get(2)?,
                    model: row.get(3)?,
                    temperature: row.get(4)?,
                    system_prompt: row.get(5)?,
                    web_search: web_search != 0,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(presets)
    }

    /// Get a single AI preset by ID
    pub fn get_ai_preset_by_id(
        &self,
        id: &str,
    ) -> Result<Option<crate::ai_presets::AiPreset>, AppError> {
        let conn = self.store.conn();
        let mut stmt = conn.prepare(
            "SELECT id, name, icon, model, temperature, system_prompt, web_search, created_at, updated_at FROM ai_presets WHERE id = ?1"
        )?;

        let result = stmt
            .query_row(params![id], |row| {
                let web_search: i32 = row.get(6)?;
                Ok(crate::ai_presets::AiPreset {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    icon: row.get(2)?,
                    model: row.get(3)?,
                    temperature: row.get(4)?,
                    system_prompt: row.get(5)?,
                    web_search: web_search != 0,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })
            .ok();

        Ok(result)
    }
}

#[tauri::command]
pub fn get_ai_usage_history(
    manager: State<AiUsageManager>,
    limit: u32,
    offset: u32,
) -> Result<Vec<GenerationData>, String> {
    manager
        .get_history(limit, offset)
        .map_err(|e| e.to_string())
}

async fn fetch_and_log_usage(
    open_router_request_id: String,
    api_key: String,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let manager = app_handle.state::<AiUsageManager>();
    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "https://openrouter.ai/api/v1/generation?id={}",
            open_router_request_id
        ))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| AppError::Ai(e.to_string()))?;

    if response.status().is_success() {
        let generation_response: Value = response
            .json()
            .await
            .map_err(|e| AppError::Ai(e.to_string()))?;
        let generation_data: GenerationData =
            serde_json::from_value(generation_response["data"].clone())
                .map_err(|e| AppError::Ai(format!("Failed to parse generation data: {}", e)))?;
        manager.log_generation(&generation_data)?;
    } else {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::Ai(format!(
            "Failed to fetch usage data: {}",
            error_text
        )));
    }
    Ok(())
}

#[tauri::command]
pub async fn get_ollama_models(base_url: String) -> Result<Vec<String>, String> {
    let client = reqwest::Client::new();
    let base = if base_url.trim().is_empty() {
        "http://localhost:11434/v1".to_string()
    } else {
        base_url
    };
    let url = format!("{}/models", base.trim_end_matches('/'));

    let res = client.get(&url).send().await.map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Failed to fetch models: {}", res.status()));
    }

    let json: Value = res.json().await.map_err(|e| e.to_string())?;

    // Ollama's /v1/models returns an object with a "data" array of models
    // Each model has an "id" field in the OpenAI-compatible API
    let mut model_ids = Vec::new();
    if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
        for model in data {
            if let Some(id) = model.get("id").and_then(|id| id.as_str()) {
                model_ids.push(id.to_string());
            }
        }
    } else {
        // Fallback for Ollama's native API /api/tags if /v1/models fails or is different
        // But since we are using /v1 base_url, /v1/models is preferred
        return Err("Unexpected response format from Ollama models API".to_string());
    }

    Ok(model_ids)
}

// Conversation Management Commands

#[tauri::command]
pub fn create_conversation(
    app_handle: AppHandle,
    title: String,
    model: Option<String>,
) -> Result<Conversation, String> {
    let usage_manager = app_handle.state::<AiUsageManager>();
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    let conversation = Conversation {
        id: id.clone(),
        title,
        created_at: now,
        updated_at: now,
        model,
        messages: Vec::new(),
    };

    let messages_json = serde_json::to_string(&conversation.messages).map_err(|e| e.to_string())?;

    usage_manager.store.execute(
        "INSERT INTO ai_conversations (id, title, created_at, updated_at, model, messages) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            conversation.id,
            conversation.title,
            conversation.created_at,
            conversation.updated_at,
            conversation.model,
            messages_json
        ],
    ).map_err(|e| e.to_string())?;

    Ok(conversation)
}

#[tauri::command]
pub fn list_conversations(app_handle: AppHandle) -> Result<Vec<Conversation>, String> {
    let usage_manager = app_handle.state::<AiUsageManager>();

    let conn = usage_manager.store.conn();
    let mut stmt = conn
        .prepare("SELECT id, title, created_at, updated_at, model, messages FROM ai_conversations ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;

    let conversations = stmt
        .query_map([], |row| {
            let messages_json: String = row.get(5)?;
            let messages: Vec<Message> =
                serde_json::from_str(&messages_json).unwrap_or_else(|_| Vec::new());

            Ok(Conversation {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                model: row.get(4)?,
                messages,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(conversations)
}

#[tauri::command]
pub fn get_conversation(app_handle: AppHandle, id: String) -> Result<Option<Conversation>, String> {
    let usage_manager = app_handle.state::<AiUsageManager>();

    let conn = usage_manager.store.conn();
    let mut stmt = conn
        .prepare("SELECT id, title, created_at, updated_at, model, messages FROM ai_conversations WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let result = stmt.query_row([id], |row| {
        let messages_json: String = row.get(5)?;
        let messages: Vec<Message> =
            serde_json::from_str(&messages_json).unwrap_or_else(|_| Vec::new());

        Ok(Conversation {
            id: row.get(0)?,
            title: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
            model: row.get(4)?,
            messages,
        })
    });

    match result {
        Ok(conv) => Ok(Some(conv)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn update_conversation(
    app_handle: AppHandle,
    id: String,
    title: Option<String>,
    messages: Option<Vec<Message>>,
) -> Result<(), String> {
    let usage_manager = app_handle.state::<AiUsageManager>();
    let now = chrono::Utc::now().timestamp();

    if let Some(msgs) = messages {
        let messages_json = serde_json::to_string(&msgs).map_err(|e| e.to_string())?;
        usage_manager
            .store
            .execute(
                "UPDATE ai_conversations SET messages = ?1, updated_at = ?2 WHERE id = ?3",
                params![messages_json, now, id],
            )
            .map_err(|e| e.to_string())?;
    }

    if let Some(t) = title {
        usage_manager
            .store
            .execute(
                "UPDATE ai_conversations SET title = ?1, updated_at = ?2 WHERE id = ?3",
                params![t, now, id],
            )
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn delete_conversation(app_handle: AppHandle, id: String) -> Result<(), String> {
    let usage_manager = app_handle.state::<AiUsageManager>();

    usage_manager
        .store
        .execute("DELETE FROM ai_conversations WHERE id = ?1", params![id])
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ai_ask_stream(
    app_handle: AppHandle,
    request_id: String,
    prompt: String,
    options: AskOptions,
) -> Result<(), String> {
    let settings = get_ai_settings(app_handle.clone())?;
    if !settings.enabled {
        return Err("AI features are not enabled.".to_string());
    }

    let api_key = if settings.provider == AiProvider::OpenRouter {
        match get_keyring_entry().and_then(|entry| entry.get_password().map_err(AppError::from)) {
            Ok(key) => key,
            Err(e) => return Err(e.to_string()),
        }
    } else {
        String::new() // Ollama doesn't need an API key
    };

    let model_key = options.model.unwrap_or_else(|| "default".to_string());

    // If a specific model ID was provided (not just "default"), use it directly
    // Otherwise, look up from model associations or fall back to default
    let model_id = if model_key != "default" && model_key.contains('/') || model_key.contains(':') {
        // Looks like a specific model ID (e.g., "openai/gpt-4o" or "llama3:latest")
        model_key.clone()
    } else {
        settings
            .model_associations
            .get(&model_key)
            .cloned()
            .unwrap_or_else(|| match settings.provider {
                AiProvider::OpenRouter => "mistralai/mistral-7b-instruct:free".to_string(),
                AiProvider::Ollama => "llama3".to_string(),
            })
    };

    // Check if tools should be enabled
    let use_tools = options.enable_tools
        && settings.tools_enabled
        && crate::ai_tools::model_supports_tools(&model_id);

    // Use configured temperature, allow creativity parameter to override if provided
    let temperature = match options.creativity.as_deref() {
        Some("none") => 0.0,
        Some("low") => 0.4,
        Some("medium") => 0.7,
        Some("high") => 1.0,
        _ => settings.temperature,
    };

    // Build initial messages
    let mut messages: Vec<serde_json::Value> =
        vec![serde_json::json!({"role": "user", "content": prompt})];

    // Build request body
    let mut body = serde_json::json!({
        "model": model_id,
        "messages": messages.clone(),
        "stream": true,
        "temperature": temperature,
    });

    // Add tools if enabled
    if use_tools {
        let tool_defs = crate::ai_tools::get_tool_definitions();
        body["tools"] = serde_json::to_value(&tool_defs).unwrap_or_default();
        tracing::info!(model = %model_id, "Tools enabled for request");
    } else if options.enable_tools {
        // User wanted tools but they're not available
        tracing::warn!(model = %model_id,
            tools_enabled = settings.tools_enabled,
            model_supports = crate::ai_tools::model_supports_tools(&model_id),
            "Tool use requested but not available"
        );
    }

    let (api_url, auth_header) = match settings.provider {
        AiProvider::OpenRouter => (
            "https://openrouter.ai/api/v1/chat/completions".to_string(),
            Some(format!("Bearer {}", api_key)),
        ),
        AiProvider::Ollama => {
            let base = settings
                .base_url
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "http://localhost:11434/v1".to_string());
            (
                format!("{}/chat/completions", base.trim_end_matches('/')),
                None,
            )
        }
    };

    let client = reqwest::Client::new();

    // Tool calling loop - may need multiple rounds
    let max_tool_rounds = 10;
    let mut tool_round = 0;

    loop {
        tool_round += 1;
        tracing::info!(round = tool_round, "Starting tool round");
        if tool_round > max_tool_rounds {
            tracing::warn!("Max tool rounds exceeded, stopping");
            break;
        }

        // Update body with current messages
        body["messages"] = serde_json::to_value(&messages).unwrap_or_default();

        let mut request = client.post(&api_url).json(&body);

        if let Some(ref auth) = auth_header {
            request = request.header("Authorization", auth.clone());
            request = request.header("HTTP-Referer", "http://localhost");
        }

        let res = request.send().await.map_err(|e| e.to_string())?;

        let open_router_request_id = res
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        if !res.status().is_success() {
            let error_body = res.text().await.unwrap_or_else(|_| "Unknown error".into());
            return Err(format!("API Error: {}", error_body));
        }

        tracing::info!("API request sent, waiting for response stream");

        let mut stream = res.bytes_stream();
        let mut full_text = String::new();
        let mut tool_calls: Vec<serde_json::Value> = Vec::new();
        let mut stream_done = false;

        while let Some(item) = stream.next().await {
            if stream_done {
                break;
            }
            let chunk = item.map_err(|e| e.to_string())?;
            let lines = String::from_utf8_lossy(&chunk);

            for line in lines.split('\n').filter(|s| !s.is_empty()) {
                if line.starts_with("data: ") {
                    let json_str = &line[6..];
                    if json_str.trim() == "[DONE]" {
                        stream_done = true;
                        break;
                    }
                    if let Ok(json) = serde_json::from_str::<Value>(json_str) {
                        // Check for finish_reason to detect stream end
                        if let Some(finish_reason) = json
                            .get("choices")
                            .and_then(|c| c.get(0))
                            .and_then(|c0| c0.get("finish_reason"))
                            .and_then(|f| f.as_str())
                        {
                            if finish_reason == "stop" || finish_reason == "tool_calls" {
                                tracing::debug!(finish_reason = %finish_reason, "Stream finished");
                                stream_done = true;
                            }
                        }

                        if let Some(delta) = json
                            .get("choices")
                            .and_then(|c| c.get(0))
                            .and_then(|c0| c0.get("delta"))
                        {
                            // Handle text content
                            if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                full_text.push_str(content);
                                app_handle
                                    .emit(
                                        "ai-stream-chunk",
                                        StreamChunk {
                                            request_id: request_id.clone(),
                                            text: content.to_string(),
                                        },
                                    )
                                    .map_err(|e| e.to_string())?;
                            }

                            // Handle tool calls (streaming)
                            if let Some(tc) = delta.get("tool_calls").and_then(|t| t.as_array()) {
                                for tool_call in tc {
                                    let index = tool_call
                                        .get("index")
                                        .and_then(|i| i.as_u64())
                                        .unwrap_or(0)
                                        as usize;

                                    // Initialize tool call if needed
                                    while tool_calls.len() <= index {
                                        tool_calls.push(serde_json::json!({
                                            "id": "",
                                            "type": "function",
                                            "function": {
                                                "name": "",
                                                "arguments": ""
                                            }
                                        }));
                                    }

                                    // Update tool call id
                                    if let Some(id) = tool_call.get("id").and_then(|i| i.as_str()) {
                                        tool_calls[index]["id"] = serde_json::json!(id);
                                    }

                                    // Update function name and arguments
                                    if let Some(func) = tool_call.get("function") {
                                        if let Some(name) =
                                            func.get("name").and_then(|n| n.as_str())
                                        {
                                            tool_calls[index]["function"]["name"] =
                                                serde_json::json!(name);
                                        }
                                        // Append arguments (they come in chunks)
                                        if let Some(args) =
                                            func.get("arguments").and_then(|a| a.as_str())
                                        {
                                            let current = tool_calls[index]["function"]
                                                ["arguments"]
                                                .as_str()
                                                .unwrap_or("");
                                            tool_calls[index]["function"]["arguments"] =
                                                serde_json::json!(format!("{}{}", current, args));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Log usage for OpenRouter
        if settings.provider == AiProvider::OpenRouter {
            if let Some(or_req_id) = open_router_request_id {
                let handle_clone = app_handle.clone();
                let key_clone = api_key.clone();
                tokio::spawn(async move {
                    if let Err(e) = fetch_and_log_usage(or_req_id, key_clone, handle_clone).await {
                        tracing::error!(error = %e, "AI usage tracking failed");
                    }
                });
            }
        }

        // If no tool calls, we're done
        tracing::info!(
            tool_count = tool_calls.len(),
            text_len = full_text.len(),
            "Stream complete"
        );
        if tool_calls.is_empty() {
            app_handle
                .emit(
                    "ai-stream-end",
                    StreamEnd {
                        request_id: request_id.clone(),
                        full_text: full_text.clone(),
                    },
                )
                .map_err(|e| e.to_string())?;
            break;
        }

        // Process tool calls
        tracing::info!(count = tool_calls.len(), "Processing tool calls");

        // Debug log each tool call
        for tc in &tool_calls {
            let name = tc
                .get("function")
                .and_then(|f| f.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("unknown");
            tracing::debug!(tool = %name, "Processing tool call");
        }

        // Add assistant message with tool calls
        messages.push(serde_json::json!({
            "role": "assistant",
            "content": if full_text.is_empty() { serde_json::Value::Null } else { serde_json::json!(full_text) },
            "tool_calls": tool_calls.clone()
        }));

        // Execute each tool call
        for tc in &tool_calls {
            let tool_call_id = tc.get("id").and_then(|i| i.as_str()).unwrap_or("");
            let tool_name = tc
                .get("function")
                .and_then(|f| f.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("");
            let arguments_str = tc
                .get("function")
                .and_then(|f| f.get("arguments"))
                .and_then(|a| a.as_str())
                .unwrap_or("{}");

            let arguments: serde_json::Value =
                serde_json::from_str(arguments_str).unwrap_or(serde_json::json!({}));

            // Get tool safety
            let tool = crate::ai_tools::BuiltinTool::from_name(tool_name);
            let safety = tool
                .map(|t| t.safety())
                .unwrap_or(crate::ai_tools::ToolSafety::Dangerous);
            let safety_str = match safety {
                crate::ai_tools::ToolSafety::Safe => "safe",
                crate::ai_tools::ToolSafety::Dangerous => "dangerous",
            };

            // Emit tool call request event
            app_handle
                .emit(
                    "ai-tool-call",
                    ToolCallRequest {
                        request_id: request_id.clone(),
                        tool_call_id: tool_call_id.to_string(),
                        tool_name: tool_name.to_string(),
                        arguments: arguments.clone(),
                        safety: safety_str.to_string(),
                    },
                )
                .map_err(|e| e.to_string())?;

            // Execute the tool (for now, auto-execute based on settings)
            // In the future, dangerous tools should wait for confirmation
            let should_execute = settings.auto_approve_all_tools
                || (settings.auto_approve_safe_tools
                    && safety == crate::ai_tools::ToolSafety::Safe);

            let tool_result = if should_execute {
                crate::ai_tools::execute_tool(tool_name, &arguments, &settings.allowed_directories)
            } else {
                Err(format!(
                    "Tool '{}' requires user confirmation (not yet implemented)",
                    tool_name
                ))
            };

            let (success, output, error) = match tool_result {
                Ok(out) => (true, out, None),
                Err(e) => (false, String::new(), Some(e)),
            };

            // Emit tool result event
            app_handle
                .emit(
                    "ai-tool-result",
                    ToolCallResult {
                        request_id: request_id.clone(),
                        tool_call_id: tool_call_id.to_string(),
                        tool_name: tool_name.to_string(),
                        success,
                        output: output.clone(),
                        error: error.clone(),
                    },
                )
                .map_err(|e| e.to_string())?;

            // Add tool result to messages
            messages.push(serde_json::json!({
                "role": "tool",
                "tool_call_id": tool_call_id,
                "content": if success { output } else { error.unwrap_or_default() }
            }));
        }

        // Continue loop for next API call with tool results
    }

    Ok(())
}
