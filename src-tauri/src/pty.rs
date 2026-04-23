use portable_pty::{CommandBuilder, NativePtySystem, PtyPair, PtySize, PtySystem};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tauri::ipc::Channel;

type WriterBox = Box<dyn Write + Send>;

struct PtySession {
    pair: Arc<Mutex<PtyPair>>,
    #[allow(dead_code)]
    child: Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>,
    writer: Arc<Mutex<Option<WriterBox>>>,
    cursor_path: PathBuf,
}

static PTY_MAP: Mutex<Option<HashMap<String, PtySession>>> = Mutex::new(None);

fn get_pty_map() -> std::sync::MutexGuard<'static, Option<HashMap<String, PtySession>>> {
    let mut map = PTY_MAP.lock().unwrap();
    if map.is_none() {
        *map = Some(HashMap::new());
    }
    map
}

fn resolve_nvim() -> String {
    let candidates = [
        "nvim",
        "/opt/homebrew/bin/nvim",
        "/usr/local/bin/nvim",
        "/usr/bin/nvim",
        "vim",
        "/opt/homebrew/bin/vim",
        "/usr/local/bin/vim",
        "C:\\Program Files\\Neovim\\bin\\nvim.exe",
        "C:\\tools\\neovim\\Neovim\\bin\\nvim.exe",
    ];
    for c in &candidates {
        if which::which(c).is_ok() {
            return c.to_string();
        }
    }
    "nvim".to_string()
}

fn write_lua_cursor_plugin(path: &PathBuf, cursor_file: &PathBuf) -> Result<(), String> {
    let lua = format!(
        r#"local cursor_file = "{}"
local last_line = 0
local function report()
  local line = vim.fn.line(".")
  if line == last_line then return end
  last_line = line
  local f = io.open(cursor_file, "w")
  if f then
    f:write(tostring(line) .. "\n")
    f:close()
  end
end
vim.api.nvim_create_autocmd({{"CursorMoved", "CursorMovedI", "BufReadPost"}}, {{
  callback = report,
}})
report()
"#,
        cursor_file.to_string_lossy().replace('\\', "\\\\").replace('"', "\\\"")
    );
    std::fs::write(path, lua).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pty_create(
    app: AppHandle,
    id: String,
    file_path: String,
    cols: u16,
    rows: u16,
    on_data: Channel<Vec<u8>>,
) -> Result<(), String> {
    let pty_system = NativePtySystem::default();

    let pair = pty_system
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;

    let nvim = resolve_nvim();
    let is_nvim = nvim.contains("nvim");

    let safe_id = id.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    let cursor_path = std::env::temp_dir().join(format!("synaptic_cursor_{}.txt", safe_id));
    let _ = std::fs::write(&cursor_path, "1\n");

    let mut cmd = CommandBuilder::new(&nvim);
    cmd.arg("-n");
    cmd.arg("-c");
    cmd.arg("set noswapfile nobackup nowritebackup updatetime=100 wrap linebreak");
    cmd.arg("-c");
    cmd.arg("autocmd TextChanged,TextChangedI <buffer> silent write");
    cmd.arg("-c");
    cmd.arg("cnoremap q <Nop> | cnoremap wq <Nop> | cnoremap x <Nop> | nnoremap ZQ <Nop> | nnoremap ZZ <Nop>");

    if is_nvim {
        let lua_path = std::env::temp_dir().join(format!("synaptic_cursor_plugin_{}.lua", safe_id));
        let _ = write_lua_cursor_plugin(&lua_path, &cursor_path);
        cmd.arg("--cmd");
        cmd.arg(format!("luafile {}", lua_path.to_string_lossy()));
    } else {
        // fallback for plain vim — use a simpler approach
        let cursor_str = cursor_path.to_string_lossy().replace('\\', "\\\\").replace("'", "''");
        cmd.arg("-c");
        cmd.arg(format!(
            "augroup SynapticCursor | autocmd! | autocmd CursorMoved,CursorMovedI,BufReadPost * call writefile([string(line('.'))], '{}') | augroup END",
            cursor_str
        ));
    }

    cmd.arg(&file_path);
    cmd.env("TERM", "xterm-256color");

    let child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;

    // Reader thread
    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let app_emit = app.clone();
    let id_emit = id.clone();

    thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = buf[..n].to_vec();
                    if on_data.send(chunk).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        let event_name: String = format!("pty-exit:{}", id_emit)
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '/' || c == ':' || c == '_' { c } else { '_' })
            .collect();
        let _ = app_emit.emit(&event_name, ());
    });

    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;

    let session = PtySession {
        pair: Arc::new(Mutex::new(pair)),
        child: Arc::new(Mutex::new(child)),
        writer: Arc::new(Mutex::new(Some(writer))),
        cursor_path,
    };

    get_pty_map()
        .as_mut()
        .unwrap()
        .insert(id, session);

    Ok(())
}

#[tauri::command]
pub fn pty_write(id: String, data: Vec<u8>) -> Result<(), String> {
    let map = get_pty_map();
    let session = map
        .as_ref()
        .unwrap()
        .get(&id)
        .ok_or("PTY session not found")?;

    let mut lock = session.writer.lock().unwrap();
    let writer = lock.as_mut().ok_or("PTY writer unavailable")?;
    writer.write_all(&data).map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn pty_resize(id: String, cols: u16, rows: u16) -> Result<(), String> {
    let map = get_pty_map();
    let session = map
        .as_ref()
        .unwrap()
        .get(&id)
        .ok_or("PTY session not found")?;

    let pair = session.pair.lock().unwrap();
    pair.master
        .resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pty_cursor_line(id: String) -> Result<u32, String> {
    let map = get_pty_map();
    let session = map
        .as_ref()
        .unwrap()
        .get(&id)
        .ok_or("PTY session not found")?;

    let path = &session.cursor_path;
    if !path.exists() {
        return Ok(1);
    }
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let line = content.trim().parse::<u32>().unwrap_or(1);
    Ok(line.max(1))
}

#[tauri::command]
pub fn pty_close(id: String) -> Result<(), String> {
    let mut map = get_pty_map();
    if let Some(session) = map.as_mut().unwrap().remove(&id) {
        let _ = std::fs::remove_file(&session.cursor_path);
        drop(session);
    }
    Ok(())
}
