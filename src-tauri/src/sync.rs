use serde::Serialize;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use tokio::time::{interval, Duration};

use crate::settings::{get_settings, Settings};

#[derive(Serialize, Clone, Debug)]
pub struct SyncState {
	pub syncing: bool,
	pub last_error: Option<String>,
	pub last_sync: Option<i64>,
}

#[derive(Serialize, Clone, Debug)]
pub struct GitCheckResult {
	pub installed: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct RepoValidationResult {
	pub valid: bool,
	pub message: String,
}

static SYNC_STATE: Mutex<SyncState> = Mutex::new(SyncState {
	syncing: false,
	last_error: None,
	last_sync: None,
});

static BACKGROUND_TASK: Mutex<Option<tauri::async_runtime::JoinHandle<()>>> = Mutex::new(None);

pub fn get_sync_state() -> SyncState {
	SYNC_STATE.lock().unwrap().clone()
}

fn set_syncing(syncing: bool) {
	let mut state = SYNC_STATE.lock().unwrap();
	state.syncing = syncing;
}

fn set_sync_result(result: Result<(), String>) {
	let mut state = SYNC_STATE.lock().unwrap();
	state.syncing = false;
	match result {
		Ok(()) => {
			state.last_error = None;
			state.last_sync = Some(chrono::Utc::now().timestamp());
		}
		Err(e) => {
			state.last_error = Some(e);
		}
	}
}

pub fn check_git_installed() -> bool {
	which::which("git").is_ok()
}

fn parse_repo_owner_repo(repo_url: &str) -> Option<(String, String)> {
	let url = repo_url.trim();
	let url = url.strip_suffix(".git").unwrap_or(url);

	// https://github.com/owner/repo
	if let Some(rest) = url.strip_prefix("https://github.com/") {
		let parts: Vec<&str> = rest.split('/').collect();
		if parts.len() >= 2 {
			return Some((parts[0].to_string(), parts[1].to_string()));
		}
	}

	// git@github.com:owner/repo
	if let Some(rest) = url.strip_prefix("git@github.com:") {
		let parts: Vec<&str> = rest.split('/').collect();
		if parts.len() >= 2 {
			return Some((parts[0].to_string(), parts[1].to_string()));
		}
	}

	None
}

fn make_authed_url(repo_url: &str, token: &str) -> String {
	let url = repo_url.trim();
	if url.starts_with("https://") {
		format!(
			"https://{}@{}",
			token,
			url.strip_prefix("https://").unwrap_or(url)
		)
	} else {
		url.to_string()
	}
}

pub async fn validate_repo_access(token: &str, repo_url: &str) -> Result<(), String> {
	let (owner, repo) = parse_repo_owner_repo(repo_url)
		.ok_or_else(|| "Could not parse owner/repo from URL".to_string())?;

	let client = reqwest::Client::new();
	let res = client
		.get(format!("https://api.github.com/repos/{}/{}", owner, repo))
		.header("Authorization", format!("Bearer {}", token))
		.header("User-Agent", "synaptic")
		.send()
		.await
		.map_err(|e| format!("GitHub API request failed: {}", e))?;

	let status = res.status();
	if status == 401 || status == 403 {
		return Err("Invalid token or insufficient permissions".to_string());
	}
	if status == 404 {
		return Err("Repository not found or token lacks access".to_string());
	}
	if !status.is_success() {
		let body = res.text().await.unwrap_or_default();
		return Err(format!("GitHub API returned {}: {}", status, body));
	}

	Ok(())
}

fn run_git(vault_path: &str, args: &[&str]) -> Result<std::process::Output, String> {
	eprintln!("[sync] git {} in {}", args.join(" "), vault_path);
	let output = Command::new("git")
		.args(args)
		.current_dir(vault_path)
		.output()
		.map_err(|e| format!("Failed to run git: {}", e))?;

	let stdout = String::from_utf8_lossy(&output.stdout);
	let stderr = String::from_utf8_lossy(&output.stderr);
	if !stdout.is_empty() {
		eprintln!("[sync] git stdout: {}", stdout.trim());
	}
	if !stderr.is_empty() {
		eprintln!("[sync] git stderr: {}", stderr.trim());
	}
	Ok(output)
}

fn run_git_expect_success(vault_path: &str, args: &[&str]) -> Result<(), String> {
	let output = run_git(vault_path, args)?;
	if !output.status.success() {
		let stderr = String::from_utf8_lossy(&output.stderr);
		return Err(format!("git {} failed: {}", args.join(" "), stderr));
	}
	Ok(())
}

pub fn ensure_repo_initialized(
	vault_path: &str,
	repo_url: &str,
	token: &str,
	branch: &str,
) -> Result<(), String> {
	let path = Path::new(vault_path);
	let git_dir = path.join(".git");

	if !git_dir.exists() {
		run_git_expect_success(vault_path, &["init"])?;
		run_git_expect_success(vault_path, &["config", "user.email", "synaptic@local"])?;
		run_git_expect_success(vault_path, &["config", "user.name", "Synaptic"])?;
	}

	let authed_url = make_authed_url(repo_url, token);

	// Set or update remote origin
	let remote_check = run_git(vault_path, &["remote", "get-url", "origin"]);
	match remote_check {
		Ok(output) if output.status.success() => {
			run_git_expect_success(vault_path, &["remote", "set-url", "origin", &authed_url])?;
		}
		_ => {
			run_git_expect_success(vault_path, &["remote", "add", "origin", &authed_url])?;
		}
	}

	// Create or reset-and-checkout the branch so we are always on the right one
	let _ = run_git(vault_path, &["checkout", "-B", branch]);

	Ok(())
}

fn is_nothing_to_commit(output: &std::process::Output) -> bool {
	let stdout = String::from_utf8_lossy(&output.stdout);
	let stderr = String::from_utf8_lossy(&output.stderr);
	let text = format!("{} {}", stdout, stderr);
	text.contains("nothing to commit") || text.contains("nothing added to commit") || text.contains("working tree clean")
}

pub async fn sync_now(vault_path: &str, settings: &Settings, app: &AppHandle) -> Result<(), String> {
	let enabled = settings.github_sync_enabled.unwrap_or(false);
	if !enabled {
		return Ok(());
	}

	let repo_url = settings
		.github_repo_url
		.as_deref()
		.ok_or("GitHub repo URL not configured")?;
	let token = settings
		.github_token
		.as_deref()
		.ok_or("GitHub token not configured")?;
	let branch = settings.github_branch.as_deref().unwrap_or("main");

	set_syncing(true);
	let _ = app.emit("sync-status", get_sync_state());

	let result = async {
		ensure_repo_initialized(vault_path, repo_url, token, branch)?;

		// Stage all changes
		run_git_expect_success(vault_path, &["add", "-A"])?;

		// Commit (ignore "nothing to commit" — git writes it to stdout with exit code 1)
		let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
		let commit_msg = format!("synaptic auto sync {}", timestamp);
		let output = run_git(vault_path, &["commit", "-m", &commit_msg])?;
		if !output.status.success() && !is_nothing_to_commit(&output) {
			let stderr = String::from_utf8_lossy(&output.stderr);
			return Err(format!("git commit failed: {}", stderr));
		}

		// Push force
		run_git_expect_success(vault_path, &["push", "--force", "origin", branch])?;

		Ok(())
	}
	.await;

	set_sync_result(result);
	let _ = app.emit("sync-status", get_sync_state());

	match get_sync_state().last_error {
		Some(ref e) => Err(e.clone()),
		None => Ok(()),
	}
}

pub fn start_background_sync(vault_path: String, app: AppHandle) {
	// Abort any existing background task
	if let Ok(mut handle) = BACKGROUND_TASK.lock() {
		if let Some(h) = handle.take() {
			h.abort();
		}
	}

	let handle = tauri::async_runtime::spawn(async move {
		let mut ticker = interval(Duration::from_secs(300)); // 5 minutes

		loop {
			ticker.tick().await;

			let settings = match get_settings(&app) {
				Ok(s) => s,
				Err(_) => continue,
			};

			if !settings.github_sync_enabled.unwrap_or(false) {
				continue;
			}

			let path = vault_path.clone();
			let app_clone = app.clone();

			if let Err(e) = sync_now(&path, &settings, &app_clone).await {
				eprintln!("Background sync failed: {}", e);
			}
		}
	});

	if let Ok(mut h) = BACKGROUND_TASK.lock() {
		*h = Some(handle);
	}
}
