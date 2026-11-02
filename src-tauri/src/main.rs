#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::Path;
use std::sync::Mutex;

use chrono::{SecondsFormat, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::Value;
use tauri::{Manager, State};

struct WorkspaceDb(Mutex<Connection>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceRecord {
    scope: String,
    key: String,
    value: Value,
    updated_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopHealth {
    product: &'static str,
    version: &'static str,
    local_database_ready: bool,
}

fn validate_scope(scope: &str) -> Result<(), String> {
    let valid_prefix = scope.starts_with("user:") || scope.starts_with("guest:");
    if !valid_prefix || scope.trim().len() > 160 {
        return Err("Workspace scope must be a bounded user: or guest: scope.".into());
    }
    Ok(())
}

fn validate_key(key: &str) -> Result<(), String> {
    if key.trim().is_empty() || key.len() > 200 {
        return Err("Workspace key must contain 1-200 characters.".into());
    }
    Ok(())
}

fn contains_sensitive_key(value: &Value, depth: usize) -> bool {
    if depth > 12 {
        return true;
    }
    match value {
        Value::Object(map) => map.iter().any(|(key, child)| {
            let normalized = key.to_ascii_lowercase().replace("_", "").replace("-", "");
            let sensitive = [
                "password", "passwd", "token", "refreshtoken", "accesstoken",
                "secret", "apikey", "authorization", "credential", "privatekey",
                "otp", "pin", "cardnumber", "cvv",
            ]
            .iter()
            .any(|needle| normalized.contains(needle));
            sensitive || contains_sensitive_key(child, depth + 1)
        }),
        Value::Array(items) => items.iter().any(|item| contains_sensitive_key(item, depth + 1)),
        _ => false,
    }
}

fn open_workspace_database(path: &Path) -> Result<Connection, Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    connection.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;
        PRAGMA busy_timeout = 5000;

        CREATE TABLE IF NOT EXISTS workspace_records (
            scope TEXT NOT NULL,
            record_key TEXT NOT NULL,
            value_json TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            PRIMARY KEY (scope, record_key)
        );

        CREATE INDEX IF NOT EXISTS idx_workspace_records_scope_updated
            ON workspace_records(scope, updated_at DESC);

        CREATE TABLE IF NOT EXISTS workspace_schema (
            version INTEGER NOT NULL
        );

        INSERT INTO workspace_schema(version)
        SELECT 1 WHERE NOT EXISTS (SELECT 1 FROM workspace_schema);
        ",
    )?;
    Ok(connection)
}

#[tauri::command]
fn desktop_health() -> DesktopHealth {
    DesktopHealth {
        product: "RevelaCode Desktop",
        version: env!("CARGO_PKG_VERSION"),
        local_database_ready: true,
    }
}

#[tauri::command]
fn workspace_put(
    state: State<'_, WorkspaceDb>,
    scope: String,
    key: String,
    value: Value,
) -> Result<WorkspaceRecord, String> {
    validate_scope(&scope)?;
    validate_key(&key)?;

    if contains_sensitive_key(&value, 0) {
        return Err("Sensitive credentials are not accepted by the local workspace store.".into());
    }

    let value_json = serde_json::to_string(&value)
        .map_err(|_| "Workspace value could not be serialised.".to_string())?;
    if value_json.len() > 2 * 1024 * 1024 {
        return Err("Workspace record exceeds the 2 MB limit.".into());
    }

    let updated_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let connection = state.0.lock().map_err(|_| "Workspace database lock is unavailable.".to_string())?;
    connection.execute(
        "INSERT INTO workspace_records(scope, record_key, value_json, updated_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(scope, record_key) DO UPDATE SET
           value_json = excluded.value_json,
           updated_at = excluded.updated_at",
        params![scope, key, value_json, updated_at],
    ).map_err(|_| "Could not save the local workspace record.".to_string())?;

    Ok(WorkspaceRecord { scope, key, value, updated_at })
}

#[tauri::command]
fn workspace_get(
    state: State<'_, WorkspaceDb>,
    scope: String,
    key: String,
) -> Result<Option<WorkspaceRecord>, String> {
    validate_scope(&scope)?;
    validate_key(&key)?;

    let connection = state.0.lock().map_err(|_| "Workspace database lock is unavailable.".to_string())?;
    let stored: Option<(String, String)> = connection
        .query_row(
            "SELECT value_json, updated_at FROM workspace_records WHERE scope = ?1 AND record_key = ?2",
            params![scope, key],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|_| "Could not read the local workspace record.".to_string())?;

    stored.map(|(value_json, updated_at)| {
        let value = serde_json::from_str(&value_json)
            .map_err(|_| "Stored workspace record is invalid.".to_string())?;
        Ok(WorkspaceRecord { scope, key, value, updated_at })
    }).transpose()
}

#[tauri::command]
fn workspace_list(
    state: State<'_, WorkspaceDb>,
    scope: String,
) -> Result<Vec<WorkspaceRecord>, String> {
    validate_scope(&scope)?;

    let connection = state.0.lock().map_err(|_| "Workspace database lock is unavailable.".to_string())?;
    let mut statement = connection
        .prepare(
            "SELECT record_key, value_json, updated_at
             FROM workspace_records WHERE scope = ?1
             ORDER BY updated_at DESC",
        )
        .map_err(|_| "Could not prepare the local workspace query.".to_string())?;

    let rows = statement
        .query_map(params![scope], |row| {
            let key: String = row.get(0)?;
            let value_json: String = row.get(1)?;
            let updated_at: String = row.get(2)?;
            Ok((key, value_json, updated_at))
        })
        .map_err(|_| "Could not list local workspace records.".to_string())?;

    let mut records = Vec::new();
    for row in rows {
        let (key, value_json, updated_at) =
            row.map_err(|_| "Could not read a local workspace row.".to_string())?;
        let value = serde_json::from_str(&value_json)
            .map_err(|_| "A stored workspace record is invalid.".to_string())?;
        records.push(WorkspaceRecord {
            scope: scope.clone(),
            key,
            value,
            updated_at,
        });
    }
    Ok(records)
}

#[tauri::command]
fn workspace_delete(
    state: State<'_, WorkspaceDb>,
    scope: String,
    key: String,
) -> Result<bool, String> {
    validate_scope(&scope)?;
    validate_key(&key)?;

    let connection = state.0.lock().map_err(|_| "Workspace database lock is unavailable.".to_string())?;
    let changed = connection
        .execute(
            "DELETE FROM workspace_records WHERE scope = ?1 AND record_key = ?2",
            params![scope, key],
        )
        .map_err(|_| "Could not delete the local workspace record.".to_string())?;
    Ok(changed > 0)
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            fs::create_dir_all(&app_data_dir)?;
            let database_path = app_data_dir.join("workspace.sqlite3");
            let connection = open_workspace_database(&database_path)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error.to_string()))?;
            app.manage(WorkspaceDb(Mutex::new(connection)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            desktop_health,
            workspace_put,
            workspace_get,
            workspace_list,
            workspace_delete
        ])
        .run(tauri::generate_context!())
        .expect("RevelaCode Desktop failed to start");
}
