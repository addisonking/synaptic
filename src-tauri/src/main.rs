// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
// use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use uuid::Uuid;
use chrono::Utc;
use walkdir::WalkDir;

mod indexer;
mod settings;
mod pty;
mod semantic;
mod ghost;

use indexer::{build_index, get_backlinks, get_graph};
use settings::{get_settings, set_settings, Settings};
use pty::{pty_create, pty_write, pty_resize, pty_close, pty_cursor_line};
use ghost::{scan_ghost_links_cmd, preview_ghost_note_cmd, preview_ghost_note_stream_cmd, create_ghost_notes_cmd};

// ─── Types ───────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SystemInfo {
    pub name: String,
    pub uuid: String,
    pub path: String,
    pub created: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileNode>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchResult {
    pub path: String,
    pub name: String,
    pub line: usize,
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VaultConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_file: Option<String>,
}

// ─── Paths ───────────────────────────────────────────────────────────────────

fn config_dir(app: &AppHandle) -> PathBuf {
    app.path()
        .app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn recent_path(app: &AppHandle) -> PathBuf {
    config_dir(app).join("recent.json")
}

fn synaptic_dir(vault_path: &str) -> PathBuf {
    Path::new(vault_path).join(".synaptic")
}

fn manifest_path(vault_path: &str) -> PathBuf {
    synaptic_dir(vault_path).join("manifest.json")
}

fn vault_config_path(vault_path: &str) -> PathBuf {
    synaptic_dir(vault_path).join("config.json")
}

fn template_path(vault_path: &str) -> PathBuf {
    synaptic_dir(vault_path).join("template.md")
}

#[allow(dead_code)]
fn index_path(vault_path: &str) -> PathBuf {
    synaptic_dir(vault_path).join("index.json")
}

// ─── Vault Lifecycle ─────────────────────────────────────────────────────────

#[tauri::command]
fn system_open(app: AppHandle, path: String) -> Result<SystemInfo, String> {
    let vault_path = Path::new(&path);
    if !vault_path.exists() {
        return Err("Vault path does not exist".to_string());
    }

    let synaptic = synaptic_dir(&path);
    let manifest = manifest_path(&path);

    let info = if manifest.exists() {
        let content = fs::read_to_string(&manifest).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())?
    } else {
        fs::create_dir_all(&synaptic).map_err(|e| e.to_string())?;
        let info = SystemInfo {
            name: vault_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Vault")
                .to_string(),
            uuid: Uuid::new_v4().to_string(),
            path: path.clone(),
            created: Utc::now().timestamp(),
        };
        let json = serde_json::to_string_pretty(&info).map_err(|e| e.to_string())?;
        fs::write(&manifest, json).map_err(|e| e.to_string())?;

        // Create default config
        let config = VaultConfig { last_file: None };
        fs::write(vault_config_path(&path), serde_json::to_string_pretty(&config).unwrap())
            .map_err(|e| e.to_string())?;

        info
    };

    // Add to recents
    add_recent_vault(&app, &info)?;

    // Create notes dir and welcome if empty
    let notes_dir = vault_path.join("notes");
    if !notes_dir.exists() {
        fs::create_dir_all(&notes_dir).ok();
    }
    if notes_dir.read_dir().map(|mut d| d.next().is_none()).unwrap_or(true) {
        let welcome_path = notes_dir.join("welcome.md");
        if !welcome_path.exists() {
            fs::write(&welcome_path, "# Welcome to Synaptic\n\nThis is your first note. Edit it in the terminal pane.\n").ok();
        }
    }

    // Auto-build semantic index in background if missing or model changed
    let path_clone = path.clone();
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        let should_rebuild = if let Some(current_model) = semantic::semantic_index_model(&path_clone) {
            let settings = get_settings(&app_clone).unwrap_or_default();
            let expected = settings.ollama_model.unwrap_or_else(|| "nomic-embed-text".to_string());
            current_model != expected
        } else {
            true
        };
        if should_rebuild {
            if let Err(e) = semantic::semantic_index_rebuild(&path_clone, &app_clone).await {
                eprintln!("Auto semantic index rebuild failed: {}", e);
            }
        }
    });

    Ok(info)
}

#[tauri::command]
fn system_list_recent(app: AppHandle) -> Result<Vec<SystemInfo>, String> {
    let path = recent_path(&app);
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut recents: Vec<SystemInfo> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    recents.retain(|r| Path::new(&r.path).exists());
    Ok(recents)
}

fn add_recent_vault(app: &AppHandle, info: &SystemInfo) -> Result<(), String> {
    let path = recent_path(app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut recents: Vec<SystemInfo> = if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        vec![]
    };
    recents.retain(|r| r.path != info.path);
    recents.insert(0, info.clone());
    recents.truncate(20);
    let json = serde_json::to_string_pretty(&recents).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn vault_create(parent: String, name: String) -> Result<String, String> {
    let path = Path::new(&parent).join(&name);
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn vault_set_last_file(system_path: String, file_path: String) -> Result<(), String> {
    let config = vault_config_path(&system_path);
    let mut cfg: VaultConfig = if config.exists() {
        let content = fs::read_to_string(&config).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or(VaultConfig { last_file: None })
    } else {
        VaultConfig { last_file: None }
    };
    cfg.last_file = Some(file_path);
    fs::write(&config, serde_json::to_string_pretty(&cfg).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn vault_get_config(system_path: String) -> Result<VaultConfig, String> {
    let config = vault_config_path(&system_path);
    if !config.exists() {
        return Ok(VaultConfig { last_file: None });
    }
    let content = fs::read_to_string(&config).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

// ─── File Tree ───────────────────────────────────────────────────────────────

#[tauri::command]
fn file_tree(system_path: String) -> Result<Vec<FileNode>, String> {
    let root = Path::new(&system_path);
    fn read_dir(path: &Path, system_path: &str) -> Result<Vec<FileNode>, String> {
        let mut entries = vec![];
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name.ends_with("~") || name.ends_with(".swp") || name.ends_with(".swo") {
                continue;
            }
            let entry_path = entry.path();
            let is_dir = entry_path.is_dir();
            let _rel_path = entry_path.strip_prefix(system_path).unwrap_or(&entry_path).to_string_lossy().to_string();
            let full_path = entry_path.to_string_lossy().to_string();

            if is_dir {
                let children = read_dir(&entry_path, system_path)?;
                if !children.is_empty() {
                    entries.push(FileNode {
                        name,
                        path: full_path,
                        is_directory: true,
                        children: Some(children),
                    });
                }
            } else if name.ends_with(".md") {
                entries.push(FileNode {
                    name,
                    path: full_path,
                    is_directory: false,
                    children: None,
                });
            }
        }
        entries.sort_by(|a, b| {
            match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
        Ok(entries)
    }
    read_dir(root, &system_path)
}

// ─── File I/O ────────────────────────────────────────────────────────────────

#[tauri::command]
fn file_read(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn file_write(path: String, content: String) -> Result<(), String> {
    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn file_create(path: String) -> Result<(), String> {
    if std::path::Path::new(&path).exists() {
        return Ok(());
    }
    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let stem = Path::new(&path).file_stem().and_then(|s| s.to_str()).unwrap_or("Note");
    fs::write(&path, format!("# {}\n\n", stem)).map_err(|e| e.to_string())
}

fn find_vault_root(file_path: &Path) -> Option<PathBuf> {
    let mut current = file_path.parent()?;
    loop {
        if current.join(".synaptic").is_dir() {
            return Some(current.to_path_buf());
        }
        current = current.parent()?;
    }
}

#[tauri::command]
fn file_delete(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err("File does not exist".to_string());
    }

    let vault_root = find_vault_root(p)
        .ok_or("Could not determine vault root for deleted file")?;
    let deleted_dir = vault_root.join(".deleted");
    fs::create_dir_all(&deleted_dir).map_err(|e| e.to_string())?;

    let file_name = p.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let mut dest = deleted_dir.join(file_name);

    // Handle collisions by appending a timestamp
    if dest.exists() {
        let stem = Path::new(file_name).file_stem().and_then(|s| s.to_str()).unwrap_or("note");
        let ext = Path::new(file_name).extension().and_then(|s| s.to_str()).unwrap_or("");
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let new_name = if ext.is_empty() {
            format!("{}_{}", stem, timestamp)
        } else {
            format!("{}_{}.{}", stem, timestamp, ext)
        };
        dest = deleted_dir.join(new_name);
    }

    fs::rename(p, dest).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn file_rename(old_path: String, new_path: String) -> Result<(), String> {
    fs::rename(&old_path, &new_path).map_err(|e| e.to_string())
}

// ─── Search ──────────────────────────────────────────────────────────────────

#[tauri::command]
fn search(system_path: String, query: String) -> Result<Vec<SearchResult>, String> {
    let mut results = vec![];
    let query_lower = query.to_lowercase();
    for entry in WalkDir::new(&system_path)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.') && !name.ends_with("~")
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !path.is_file() || ext != "md" {
            continue;
        }
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for (line_num, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(&query_lower) {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                results.push(SearchResult {
                    path: path.to_string_lossy().to_string(),
                    name,
                    line: line_num + 1,
                    content: line.to_string(),
                });
                if results.len() >= 100 {
                    return Ok(results);
                }
            }
        }
    }
    Ok(results)
}

// ─── Semantic Search Commands ────────────────────────────────────────────────

#[tauri::command]
async fn semantic_index_rebuild_cmd(system_path: String, app: AppHandle) -> Result<(), String> {
    semantic::semantic_index_rebuild(&system_path, &app).await
}

#[tauri::command]
async fn semantic_search_cmd(
    system_path: String,
    query: String,
    app: AppHandle,
) -> Result<Vec<semantic::SemanticResult>, String> {
    semantic::semantic_search(&system_path, &query, 20, &app).await
}

#[tauri::command]
async fn test_ollama_connection_cmd(app: AppHandle) -> Result<semantic::OllamaHealth, String> {
    semantic::test_ollama_connection(&app).await
}

// ─── Indexer Commands ────────────────────────────────────────────────────────

#[tauri::command]
fn index_rebuild(system_path: String) -> Result<(), String> {
    build_index(&system_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_backlinks_cmd(system_path: String, note_name: String) -> Result<Vec<indexer::BacklinkInfo>, String> {
    get_backlinks(&system_path, &note_name).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_graph_cmd(system_path: String) -> Result<indexer::GraphData, String> {
    get_graph(&system_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn find_note(system_path: String, name: String) -> Result<Option<String>, String> {
    let target_lower = name.to_lowercase();
    for entry in WalkDir::new(&system_path)
        .into_iter()
        .filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !path.is_file() || ext != "md" {
            continue;
        }
        let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        if file_name == target_lower {
            return Ok(Some(path.to_string_lossy().to_string()));
        }
    }
    Ok(None)
}

// ─── Settings Commands ───────────────────────────────────────────────────────

#[tauri::command]
fn get_settings_cmd(app: AppHandle) -> Result<Settings, String> {
    get_settings(&app).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_settings_cmd(app: AppHandle, settings: Settings) -> Result<(), String> {
    set_settings(&app, settings).map_err(|e| e.to_string())
}

// ─── Main ────────────────────────────────────────────────────────────────────

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            system_open,
            system_list_recent,
            vault_create,
            vault_set_last_file,
            vault_get_config,
            file_tree,
            file_read,
            file_write,
            file_create,
            file_delete,
            file_rename,
            search,
            index_rebuild,
            get_backlinks_cmd,
            get_graph_cmd,
            find_note,
            get_settings_cmd,
            set_settings_cmd,
            semantic_index_rebuild_cmd,
            semantic_search_cmd,
            test_ollama_connection_cmd,
            pty_create,
            pty_write,
            pty_resize,
            pty_cursor_line,
            pty_close,
            scan_ghost_links_cmd,
            preview_ghost_note_cmd,
            preview_ghost_note_stream_cmd,
            create_ghost_notes_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
