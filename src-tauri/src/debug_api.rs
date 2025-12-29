// HTTP Debug API for development and MCP integration
// Runs on 127.0.0.1:7266 alongside the browser extension WebSocket server

use axum::{
    extract::{Query, State},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Manager};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::Layer;

// ============================================================================
// Log Buffer - Ring buffer for storing recent log entries
// ============================================================================

const MAX_LOG_ENTRIES: usize = 1000;

#[derive(Clone, Serialize, Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
    pub fields: Option<Value>,
}

/// Thread-safe ring buffer for log entries
pub struct LogBuffer {
    entries: RwLock<VecDeque<LogEntry>>,
}

impl LogBuffer {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(VecDeque::with_capacity(MAX_LOG_ENTRIES)),
        }
    }

    pub fn push(&self, entry: LogEntry) {
        let mut entries = self.entries.write().unwrap();
        if entries.len() >= MAX_LOG_ENTRIES {
            entries.pop_front();
        }
        entries.push_back(entry);
    }

    pub fn get_entries(
        &self,
        limit: Option<usize>,
        level: Option<&str>,
        search: Option<&str>,
    ) -> Vec<LogEntry> {
        let entries = self.entries.read().unwrap();
        let mut result: Vec<LogEntry> = entries
            .iter()
            .filter(|e| {
                // Filter by level if specified
                if let Some(lvl) = level {
                    if !e.level.eq_ignore_ascii_case(lvl) {
                        return false;
                    }
                }
                // Filter by search term if specified
                if let Some(term) = search {
                    let term_lower = term.to_lowercase();
                    if !e.message.to_lowercase().contains(&term_lower)
                        && !e.target.to_lowercase().contains(&term_lower)
                    {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        // Return most recent entries (reverse order, then take limit)
        result.reverse();
        if let Some(n) = limit {
            result.truncate(n);
        }
        result
    }

    pub fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
        entries.clear();
    }
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self::new()
    }
}

// Global log buffer instance and configuration
lazy_static::lazy_static! {
    pub static ref LOG_BUFFER: Arc<LogBuffer> = Arc::new(LogBuffer::new());
    static ref LOG_CAPTURE_LEVEL: RwLock<tracing::Level> = RwLock::new(tracing::Level::INFO);
}

/// Get the current minimum log level for capture
pub fn get_log_capture_level() -> tracing::Level {
    *LOG_CAPTURE_LEVEL.read().unwrap()
}

/// Set the minimum log level for capture
pub fn set_log_capture_level(level: tracing::Level) {
    *LOG_CAPTURE_LEVEL.write().unwrap() = level;
}

/// Parse a log level string into a tracing::Level
pub fn parse_log_level(s: &str) -> Option<tracing::Level> {
    match s.to_lowercase().as_str() {
        "trace" => Some(tracing::Level::TRACE),
        "debug" => Some(tracing::Level::DEBUG),
        "info" => Some(tracing::Level::INFO),
        "warn" | "warning" => Some(tracing::Level::WARN),
        "error" => Some(tracing::Level::ERROR),
        _ => None,
    }
}

/// Custom tracing layer that captures logs to the ring buffer
pub struct LogCaptureLayer;

impl<S> Layer<S> for LogCaptureLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();

        // Check if this log level should be captured
        let min_level = get_log_capture_level();
        if metadata.level() > &min_level {
            return;
        }

        // Extract message and fields
        let mut visitor = FieldVisitor::default();
        event.record(&mut visitor);

        let entry = LogEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            level: metadata.level().to_string(),
            target: metadata.target().to_string(),
            message: visitor.message,
            fields: if visitor.fields.is_empty() {
                None
            } else {
                Some(serde_json::to_value(&visitor.fields).unwrap_or(Value::Null))
            },
        };

        LOG_BUFFER.push(entry);
    }
}

#[derive(Default)]
struct FieldVisitor {
    message: String,
    fields: std::collections::HashMap<String, String>,
}

impl tracing::field::Visit for FieldVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        } else {
            self.fields
                .insert(field.name().to_string(), format!("{:?}", value));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else {
            self.fields
                .insert(field.name().to_string(), value.to_string());
        }
    }
}

use crate::aliases::AliasManager;
use crate::app::App;
use crate::cache::AppCache;
use crate::extensions;
use crate::frecency::FrecencyManager;
use crate::quicklinks::QuicklinkManager;
use crate::settings::SettingsManager;
use crate::snippets::manager::SnippetManager;

/// Shared state for the debug API
#[derive(Clone)]
pub struct DebugApiState {
    app_handle: AppHandle,
}

impl DebugApiState {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

/// Health check response
#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
    uptime_seconds: u64,
}

/// Generic API response wrapper
#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    fn ok(data: T) -> Json<Self> {
        Json(Self {
            success: true,
            data: Some(data),
            error: None,
        })
    }

    fn err(msg: impl Into<String>) -> Json<Self> {
        Json(Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        })
    }
}

// Track when the server started
static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

/// GET /health - Check if Flareup is running
async fn health() -> Json<HealthResponse> {
    let uptime = START_TIME.get().map(|t| t.elapsed().as_secs()).unwrap_or(0);

    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        uptime_seconds: uptime,
    })
}

/// GET /apps - List installed applications
async fn list_apps(State(state): State<Arc<DebugApiState>>) -> Json<ApiResponse<Vec<App>>> {
    match AppCache::get_apps(&state.app_handle) {
        Ok(apps) => ApiResponse::ok(apps),
        Err(e) => ApiResponse::err(format!("Failed to get apps: {}", e)),
    }
}

/// GET /plugins - List discovered plugins/extensions
async fn list_plugins(
    State(state): State<Arc<DebugApiState>>,
) -> Json<ApiResponse<Vec<extensions::PluginInfo>>> {
    match extensions::discover_plugins(&state.app_handle) {
        Ok(plugins) => ApiResponse::ok(plugins),
        Err(e) => ApiResponse::err(e),
    }
}

/// GET /snippets - List all snippets
async fn list_snippets(
    State(state): State<Arc<DebugApiState>>,
) -> Json<ApiResponse<Vec<crate::snippets::Snippet>>> {
    let manager = state.app_handle.state::<SnippetManager>();
    match manager.list_snippets(None) {
        Ok(snippets) => ApiResponse::ok(snippets),
        Err(e) => ApiResponse::err(format!("{}", e)),
    }
}

/// GET /quicklinks - List all quicklinks
async fn list_quicklinks(
    State(state): State<Arc<DebugApiState>>,
) -> Json<ApiResponse<Vec<crate::quicklinks::Quicklink>>> {
    let manager = state.app_handle.state::<QuicklinkManager>();
    match manager.list_quicklinks() {
        Ok(links) => ApiResponse::ok(links),
        Err(e) => ApiResponse::err(format!("{}", e)),
    }
}

/// GET /frecency - Get frecency data
async fn get_frecency(
    State(state): State<Arc<DebugApiState>>,
) -> Json<ApiResponse<Vec<crate::frecency::FrecencyData>>> {
    let manager = state.app_handle.state::<FrecencyManager>();
    match manager.get_frecency_data() {
        Ok(data) => ApiResponse::ok(data),
        Err(e) => ApiResponse::err(format!("{}", e)),
    }
}

/// GET /aliases - Get command aliases
async fn get_aliases(
    State(state): State<Arc<DebugApiState>>,
) -> Json<ApiResponse<std::collections::HashMap<String, String>>> {
    let manager = state.app_handle.state::<AliasManager>();
    match manager.get_all() {
        Ok(aliases) => ApiResponse::ok(aliases),
        Err(e) => ApiResponse::err(format!("{}", e)),
    }
}

/// GET /settings - Get app settings
async fn get_settings(
    State(state): State<Arc<DebugApiState>>,
) -> Json<ApiResponse<crate::settings::AppSettings>> {
    let manager = state.app_handle.state::<SettingsManager>();
    match manager.get_settings() {
        Ok(settings) => ApiResponse::ok(settings),
        Err(e) => ApiResponse::err(format!("{}", e)),
    }
}

/// GET /ai/settings - Get AI settings
async fn get_ai_settings(
    State(state): State<Arc<DebugApiState>>,
) -> Json<ApiResponse<crate::ai::AiSettings>> {
    match crate::ai::get_ai_settings_internal(&state.app_handle) {
        Ok(settings) => ApiResponse::ok(settings),
        Err(e) => ApiResponse::err(e),
    }
}

/// Query parameters for logs endpoint
#[derive(Deserialize, Default)]
struct LogsQuery {
    /// Maximum number of entries to return (default: 100)
    limit: Option<usize>,
    /// Filter by log level (error, warn, info, debug, trace)
    level: Option<String>,
    /// Search term to filter by message or target
    search: Option<String>,
}

/// GET /logs - Get recent log entries
async fn get_logs(Query(query): Query<LogsQuery>) -> Json<ApiResponse<Vec<LogEntry>>> {
    let limit = query.limit.or(Some(100));
    let entries = LOG_BUFFER.get_entries(limit, query.level.as_deref(), query.search.as_deref());
    ApiResponse::ok(entries)
}

/// DELETE /logs - Clear log buffer
async fn clear_logs() -> Json<ApiResponse<()>> {
    LOG_BUFFER.clear();
    ApiResponse::ok(())
}

/// Response for log config endpoint
#[derive(Serialize)]
struct LogConfigResponse {
    level: String,
    available_levels: Vec<&'static str>,
}

/// GET /logs/config - Get current log capture configuration
async fn get_log_config() -> Json<ApiResponse<LogConfigResponse>> {
    let level = get_log_capture_level();
    ApiResponse::ok(LogConfigResponse {
        level: level.to_string().to_lowercase(),
        available_levels: vec!["trace", "debug", "info", "warn", "error"],
    })
}

/// Request body for setting log config
#[derive(Deserialize)]
struct SetLogConfigRequest {
    level: String,
}

/// POST /logs/config - Set log capture level
async fn set_log_config(
    Json(body): Json<SetLogConfigRequest>,
) -> Json<ApiResponse<LogConfigResponse>> {
    match parse_log_level(&body.level) {
        Some(level) => {
            set_log_capture_level(level);
            ApiResponse::ok(LogConfigResponse {
                level: level.to_string().to_lowercase(),
                available_levels: vec!["trace", "debug", "info", "warn", "error"],
            })
        }
        None => ApiResponse::err(format!(
            "Invalid log level: '{}'. Valid levels: trace, debug, info, warn, error",
            body.level
        )),
    }
}

/// Jump mode data for debugging
#[derive(Serialize)]
struct JumpModeData {
    enabled: bool,
    fd_available: bool,
    editor_command: String,
    max_results: usize,
    search_hidden: bool,
}

/// GET /jump-mode - Get jump mode status and configuration
async fn get_jump_mode() -> Json<ApiResponse<JumpModeData>> {
    let fd_available = crate::jump_mode::is_fd_available();
    let config = crate::jump_mode::JumpModeConfig::default();
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "xdg-open".to_string());

    ApiResponse::ok(JumpModeData {
        enabled: config.enabled,
        fd_available,
        editor_command: editor,
        max_results: config.max_results,
        search_hidden: config.search_hidden,
    })
}

/// Request body for executing arbitrary commands
#[derive(Deserialize)]
struct CommandRequest {
    params: Option<Value>,
}

/// POST /command/:name - Execute a Tauri command by name (limited set for safety)
async fn execute_command(
    State(state): State<Arc<DebugApiState>>,
    axum::extract::Path(command_name): axum::extract::Path<String>,
    body: Option<Json<CommandRequest>>,
) -> Json<ApiResponse<Value>> {
    let params = body.and_then(|b| b.params.clone()).unwrap_or(json!({}));

    // Whitelist of safe read-only commands
    let result = match command_name.as_str() {
        "get_installed_apps" => {
            let apps = AppCache::get_apps(&state.app_handle)
                .map_err(|e| format!("{}", e))
                .map(|a| serde_json::to_value(a).unwrap_or(Value::Null));
            apps
        }
        "get_discovered_plugins" => extensions::discover_plugins(&state.app_handle)
            .map(|p| serde_json::to_value(p).unwrap_or(Value::Null)),
        "get_frecency_data" => {
            let manager = state.app_handle.state::<FrecencyManager>();
            manager
                .get_frecency_data()
                .map_err(|e| format!("{}", e))
                .map(|d| serde_json::to_value(d).unwrap_or(Value::Null))
        }
        "list_snippets" => {
            let manager = state.app_handle.state::<SnippetManager>();
            manager
                .list_snippets(None)
                .map_err(|e| format!("{}", e))
                .map(|s| serde_json::to_value(s).unwrap_or(Value::Null))
        }
        "list_quicklinks" => {
            let manager = state.app_handle.state::<QuicklinkManager>();
            manager
                .list_quicklinks()
                .map_err(|e| format!("{}", e))
                .map(|l| serde_json::to_value(l).unwrap_or(Value::Null))
        }
        "get_aliases" => {
            let manager = state.app_handle.state::<AliasManager>();
            manager
                .get_all()
                .map_err(|e| format!("{}", e))
                .map(|a| serde_json::to_value(a).unwrap_or(Value::Null))
        }
        "get_app_settings" => {
            let manager = state.app_handle.state::<SettingsManager>();
            manager
                .get_settings()
                .map_err(|e| format!("{}", e))
                .map(|s| serde_json::to_value(s).unwrap_or(Value::Null))
        }
        _ => Err(format!(
            "Unknown or unsafe command: {}. Available: get_installed_apps, get_discovered_plugins, get_frecency_data, list_snippets, list_quicklinks, get_aliases, get_app_settings",
            command_name
        )),
    };

    match result {
        Ok(value) => ApiResponse::ok(value),
        Err(e) => ApiResponse::err(e),
    }
}

/// GET / - API index with available endpoints
async fn index() -> Json<Value> {
    Json(json!({
        "name": "Flareup Debug API",
        "version": env!("CARGO_PKG_VERSION"),
        "endpoints": {
            "GET /": "This index",
            "GET /health": "Health check",
            "GET /apps": "List installed applications",
            "GET /plugins": "List discovered plugins",
            "GET /snippets": "List all snippets",
            "GET /quicklinks": "List all quicklinks",
            "GET /frecency": "Get frecency data",
            "GET /aliases": "Get command aliases",
            "GET /settings": "Get app settings",
            "GET /logs": "Get recent log entries (?limit=N&level=info&search=term)",
            "DELETE /logs": "Clear log buffer",
            "GET /logs/config": "Get log capture configuration",
            "POST /logs/config": "Set log capture level ({\"level\": \"info\"})",
            "GET /ai/settings": "Get AI settings",
            "GET /jump-mode": "Get jump mode configuration and status",
            "POST /command/:name": "Execute a whitelisted command"
        }
    }))
}

/// Create the debug API router
pub fn create_router(app_handle: AppHandle) -> Router {
    let state = Arc::new(DebugApiState::new(app_handle));

    // CORS layer for local development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/apps", get(list_apps))
        .route("/plugins", get(list_plugins))
        .route("/snippets", get(list_snippets))
        .route("/quicklinks", get(list_quicklinks))
        .route("/frecency", get(get_frecency))
        .route("/aliases", get(get_aliases))
        .route("/settings", get(get_settings))
        .route("/ai/settings", get(get_ai_settings))
        .route("/jump-mode", get(get_jump_mode))
        .route("/logs", get(get_logs).delete(clear_logs))
        .route("/logs/config", get(get_log_config).post(set_log_config))
        .route("/command/{name}", post(execute_command))
        .layer(cors)
        .with_state(state)
}

/// Run the debug API server
pub async fn run_server(app_handle: AppHandle) {
    // Initialize start time
    let _ = START_TIME.set(std::time::Instant::now());

    let addr = "127.0.0.1:7266";
    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("Failed to bind debug API port {}: {}", addr, e);
            tracing::warn!("Debug API will not be available");
            return;
        }
    };

    tracing::info!("Debug API server listening on http://{}", addr);

    let router = create_router(app_handle);
    if let Err(e) = axum::serve(listener, router).await {
        tracing::error!("Debug API server error: {}", e);
    }
}
