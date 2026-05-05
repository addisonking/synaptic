use notify::{Event, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

static FILE_WATCHERS: Mutex<Option<HashMap<String, notify::RecommendedWatcher>>> =
	Mutex::new(None);

fn get_watchers() -> std::sync::MutexGuard<'static, Option<HashMap<String, notify::RecommendedWatcher>>>
{
	let mut map = FILE_WATCHERS.lock().unwrap();
	if map.is_none() {
		*map = Some(HashMap::new());
	}
	map
}

#[tauri::command]
pub fn watch_file(app: AppHandle, path: String) -> Result<(), String> {
	let mut map = get_watchers();
	let map = map.as_mut().unwrap();

	// Drop any existing watcher for this path
	map.remove(&path);

	let app_emit = app.clone();
	let path_clone = path.clone();
	let mut watcher = notify::recommended_watcher(
		move |res: Result<Event, notify::Error>| {
			if let Ok(event) = res {
				if matches!(
					event.kind,
					notify::EventKind::Create(_) | notify::EventKind::Modify(_)
				) {
					let _ = app_emit.emit("file-changed", path_clone.clone());
				}
			}
		},
	)
	.map_err(|e| e.to_string())?;

	watcher
		.watch(Path::new(&path), RecursiveMode::NonRecursive)
		.map_err(|e| e.to_string())?;

	map.insert(path, watcher);
	Ok(())
}

#[tauri::command]
pub fn unwatch_file(path: String) -> Result<(), String> {
	let mut map = get_watchers();
	if let Some(map) = map.as_mut() {
		map.remove(&path);
	}
	Ok(())
}
