use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nvim_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ollama_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ollama_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_sync_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_repo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_branch: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            nvim_path: None,
            ollama_url: Some("http://localhost:11434".to_string()),
            ollama_model: Some("nomic-embed-text".to_string()),
            generation_model: Some("gemma4:26b".to_string()),
            github_sync_enabled: Some(false),
            github_repo_url: None,
            github_token: None,
            github_branch: Some("main".to_string()),
        }
    }
}

fn settings_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("settings.json")
}

pub fn config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("com.synaptic.app")
}

fn merge_defaults(settings: &mut Settings) {
    if settings.ollama_url.is_none() {
        settings.ollama_url = Settings::default().ollama_url;
    }
    if settings.ollama_model.is_none() {
        settings.ollama_model = Settings::default().ollama_model;
    }
    if settings.generation_model.is_none() {
        settings.generation_model = Settings::default().generation_model;
    }
}

pub fn get_settings(app: &AppHandle) -> Result<Settings, std::io::Error> {
    let path = settings_path(app);
    let settings = read_settings_file(&path);
    Ok(settings)
}

pub fn get_settings_cli() -> Settings {
    let path = config_dir().join("settings.json");
    read_settings_file(&path)
}

fn read_settings_file(path: &std::path::Path) -> Settings {
    if !path.exists() {
        return Settings::default();
    }
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Settings::default(),
    };
    let mut settings: Settings = serde_json::from_str(&content).unwrap_or_default();
    merge_defaults(&mut settings);
    settings
}

pub fn set_settings(app: &AppHandle, settings: Settings) -> Result<(), std::io::Error> {
    let path = settings_path(app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_string_pretty(&settings)?)?;
    Ok(())
}
