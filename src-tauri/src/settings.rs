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

pub fn get_settings(app: &AppHandle) -> Result<Settings, std::io::Error> {
    let path = settings_path(app);
    if !path.exists() {
        return Ok(Settings::default());
    }
    let content = fs::read_to_string(&path)?;
    let mut settings: Settings = serde_json::from_str(&content).unwrap_or_default();
    if settings.ollama_url.is_none() {
        settings.ollama_url = Settings::default().ollama_url;
    }
    if settings.ollama_model.is_none() {
        settings.ollama_model = Settings::default().ollama_model;
    }
    if settings.generation_model.is_none() {
        settings.generation_model = Settings::default().generation_model;
    }
    Ok(settings)
}

pub fn set_settings(app: &AppHandle, settings: Settings) -> Result<(), std::io::Error> {
    let path = settings_path(app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_string_pretty(&settings)?)?;
    Ok(())
}
