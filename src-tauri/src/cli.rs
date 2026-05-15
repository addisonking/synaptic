use clap::{Parser, Subcommand};
use serde::Serialize;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use crate::indexer;
use crate::semantic;
use crate::settings;

// ─── CLI Definition ──────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "synaptic",
    about = "Personal knowledge graph — CLI",
    version
)]
pub struct Cli {
    /// Vault path (defaults to SYNAPTIC_VAULT env or auto-detect from cwd)
    #[arg(short, long, env = "SYNAPTIC_VAULT", global = true)]
    vault: Option<String>,

    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Open (or initialize) a vault
    Open {
        path: String,
    },

    /// List recently opened vaults
    ListRecent,

    /// Show file tree [default: vault root]
    Tree {
        /// Directory to list (defaults to vault root)
        path: Option<String>,
    },

    /// Read a file's contents
    Read {
        path: String,
    },

    /// Create a new note
    Create {
        /// Path for the new note (relative to vault or absolute)
        path: String,
    },

    /// Write content to a file (from arg or stdin)
    Write {
        /// Path to write to
        path: String,
        /// Content to write (if omitted, reads from stdin)
        content: Option<String>,
    },

    /// Delete a note (moves to .deleted/)
    Delete {
        path: String,
    },

    /// Rename a note and update all backlinks
    Rename {
        old: String,
        new: String,
    },

    /// Full-text search across all notes
    Search {
        query: String,
    },

    /// Semantic (Ollama) search
    SemanticSearch {
        query: String,
    },

    /// Show backlinks for a note
    Backlinks {
        note: String,
    },

    /// Output graph data (nodes + edges)
    Graph,

    /// List all tags
    Tags,

    /// Find a note by name (case-insensitive, returns path)
    Find {
        name: String,
    },

    /// Check dependency status (nvim, ollama)
    Check,

    /// Rebuild the semantic (Ollama) search index
    RebuildIndex,

    /// Symlink the synaptic binary into /usr/local/bin or ~/.local/bin
    AddToPath,

    /// Remove the symlink created by add-to-path
    RemoveFromPath,
}

// ─── Vault Resolution ───────────────────────────────────────────────────────

fn find_vault_root(start: &Path) -> Option<PathBuf> {
    let mut current = if start.is_dir() {
        start.to_path_buf()
    } else {
        start.parent()?.to_path_buf()
    };
    loop {
        if current.join(".synaptic").is_dir() {
            return Some(current);
        }
        current = current.parent()?.to_path_buf();
    }
}

fn resolve_vault(cli: &Cli) -> Result<String, String> {
    if let Some(ref v) = cli.vault {
        let p = Path::new(v);
        if !p.exists() {
            return Err(format!("vault path does not exist: {}", v));
        }
        return Ok(p.canonicalize()
            .unwrap_or_else(|_| p.to_path_buf())
            .to_string_lossy()
            .to_string());
    }

    // auto-detect from cwd
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    find_vault_root(&cwd)
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| {
            "no vault specified (use --vault, SYNAPTIC_VAULT, or run from inside a vault)".to_string()
        })
}

fn require_vault(cli: &Cli) -> Result<String, String> {
    resolve_vault(cli).and_then(|v| {
        let p = Path::new(&v);
        if !p.join(".synaptic").is_dir() {
            return Err(format!("'{}' is not a synaptic vault (no .synaptic dir)", v));
        }
        Ok(v)
    })
}

// ─── JSON Output Helpers ────────────────────────────────────────────────────

fn print_json<T: Serialize>(val: &T) {
    println!("{}", serde_json::to_string_pretty(val).unwrap());
}

// ─── Command Handlers ───────────────────────────────────────────────────────

fn cmd_open(path: &str, json: bool) -> Result<(), String> {
    use crate::{SystemInfo, VaultConfig};
    use uuid::Uuid;
    use chrono::Utc;

    let vault_path = Path::new(path);
    if !vault_path.exists() {
        return Err(format!("path does not exist: {}", path));
    }

    let synaptic_dir = vault_path.join(".synaptic");
    let manifest = synaptic_dir.join("manifest.json");

    let info: SystemInfo = if manifest.exists() {
        let content = std::fs::read_to_string(&manifest).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())?
    } else {
        std::fs::create_dir_all(&synaptic_dir).map_err(|e| e.to_string())?;
        let info = SystemInfo {
            name: vault_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Vault")
                .to_string(),
            uuid: Uuid::new_v4().to_string(),
            path: path.to_string(),
            created: Utc::now().timestamp(),
        };
        let json_str = serde_json::to_string_pretty(&info).map_err(|e| e.to_string())?;
        std::fs::write(&manifest, json_str).map_err(|e| e.to_string())?;

        let config = VaultConfig { last_file: None };
        std::fs::write(
            synaptic_dir.join("config.json"),
            serde_json::to_string_pretty(&config).unwrap(),
        )
        .map_err(|e| e.to_string())?;

        let notes_dir = vault_path.join("notes");
        std::fs::create_dir_all(&notes_dir).ok();
        let scratch_dir = vault_path.join("scratch");
        std::fs::create_dir_all(&scratch_dir).ok();

        if notes_dir.read_dir().map(|mut d| d.next().is_none()).unwrap_or(true) {
            let welcome = notes_dir.join("welcome.md");
            if !welcome.exists() {
                std::fs::write(&welcome, "# Welcome to Synaptic\n\nThis is your first note.\n").ok();
            }
        }

        // write to recents
        add_recent_cli(&info);

        info
    };

    if json {
        print_json(&info);
    } else {
        println!("name:    {}", info.name);
        println!("uuid:    {}", info.uuid);
        println!("path:    {}", info.path);
        println!("created: {}", info.created);
    }

    Ok(())
}

fn add_recent_cli(info: &crate::SystemInfo) {
    let config_dir = settings::config_dir();
    let _ = std::fs::create_dir_all(&config_dir);
    let path = config_dir.join("recent.json");

    let mut recents: Vec<crate::SystemInfo> = if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    } else {
        vec![]
    };

    recents.retain(|r| r.path != info.path);
    recents.insert(0, info.clone());
    recents.truncate(20);

    let json = serde_json::to_string_pretty(&recents).unwrap();
    let _ = std::fs::write(&path, json);
}

fn cmd_list_recent(json: bool) -> Result<(), String> {
    let path = settings::config_dir().join("recent.json");
    if !path.exists() {
        if json {
            println!("[]");
        }
        return Ok(());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let recents: Vec<crate::SystemInfo> =
        serde_json::from_str(&content).map_err(|e| e.to_string())?;
    let recents: Vec<_> = recents
        .into_iter()
        .filter(|r| Path::new(&r.path).exists())
        .collect();

    if json {
        print_json(&recents);
    } else {
        for r in &recents {
            println!("{}  {}", r.name, r.path);
        }
    }
    Ok(())
}

fn cmd_tree(vault: &str, path: Option<&str>, json: bool) -> Result<(), String> {
    let root = match path {
        Some(p) => {
            let resolved = Path::new(p);
            if resolved.is_absolute() {
                resolved.to_path_buf()
            } else {
                Path::new(vault).join(p)
            }
        }
        None => Path::new(vault).to_path_buf(),
    };

    let tree = build_tree(&root, vault)?;

    if json {
        print_json(&tree);
    } else {
        print_tree_nodes(&tree, "");
    }
    Ok(())
}

fn build_tree(dir: &Path, vault: &str) -> Result<Vec<crate::FileNode>, String> {
    use crate::FileNode;
    let mut entries: Vec<FileNode> = vec![];

    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "scratch" || name.ends_with('~') || name.ends_with(".swp") || name.ends_with(".swo") {
            continue;
        }
        let entry_path = entry.path();
        let is_dir = entry_path.is_dir();
        let full_path = entry_path.to_string_lossy().to_string();

        if is_dir {
            let children = build_tree(&entry_path, vault)?;
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

fn print_tree_nodes(nodes: &[crate::FileNode], prefix: &str) {
    for (i, node) in nodes.iter().enumerate() {
        let is_last = i == nodes.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let child_prefix = if is_last { "    " } else { "│   " };

        if node.is_directory {
            println!("{}{}{}/", prefix, connector, node.name);
            if let Some(ref children) = node.children {
                print_tree_nodes(children, &format!("{}{}", prefix, child_prefix));
            }
        } else {
            println!("{}{}{}", prefix, connector, node.name);
        }
    }
}

fn resolve_path(vault: Option<&str>, path: &str) -> PathBuf {
    let p = Path::new(path);
    if p.is_absolute() {
        p.to_path_buf()
    } else if let Some(v) = vault {
        Path::new(v).join(path)
    } else {
        p.to_path_buf()
    }
}

fn cmd_read(vault: Option<&str>, path: &str, _json: bool) -> Result<(), String> {
    let full = resolve_path(vault, path);
    let content = std::fs::read_to_string(&full)
        .map_err(|e| format!("{}: {}", full.display(), e))?;
    print!("{}", content);
    Ok(())
}

fn cmd_create(vault: &str, path: &str) -> Result<(), String> {
    let full_path = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        Path::new(vault).join(path)
    };

    if full_path.exists() {
        return Err(format!("file already exists: {}", full_path.display()));
    }

    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let stem = full_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Note");

    std::fs::write(
        &full_path,
        format!("---\ntags: []\n---\n\n# {}\n\n", stem),
    )
    .map_err(|e| e.to_string())?;

    println!("{}", full_path.display());
    Ok(())
}

fn cmd_write(vault: &str, path: &str, content: Option<&str>) -> Result<(), String> {
    let full_path = resolve_path(Some(vault), path);
    let data = match content {
        Some(c) => c.to_string(),
        None => {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| e.to_string())?;
            buf
        }
    };

    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&full_path, &data).map_err(|e| e.to_string())?;
    Ok(())
}

fn cmd_delete(vault: &str, path: &str) -> Result<(), String> {
    let full_path = resolve_path(Some(vault), path);
    let p = &full_path;
    if !p.exists() {
        return Err("file does not exist".to_string());
    }

    let vault_root = find_vault_root(p)
        .ok_or("could not determine vault root")?;
    let deleted_dir = vault_root.join(".deleted");
    std::fs::create_dir_all(&deleted_dir).map_err(|e| e.to_string())?;

    let file_name = p
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let mut dest = deleted_dir.join(file_name);

    if dest.exists() {
        let stem = Path::new(file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("note");
        let ext = Path::new(file_name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let new_name = if ext.is_empty() {
            format!("{}_{}", stem, timestamp)
        } else {
            format!("{}_{}.{}", stem, timestamp, ext)
        };
        dest = deleted_dir.join(new_name);
    }

    std::fs::rename(p, dest).map_err(|e| e.to_string())?;
    Ok(())
}

fn cmd_rename(vault: &str, old: &str, new_name: &str) -> Result<(), String> {
    use regex::Regex;
    use walkdir::WalkDir;

    let old_path = if Path::new(old).is_absolute() {
        PathBuf::from(old)
    } else {
        Path::new(vault).join(old)
    };

    let old_stem = old_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("invalid path")?;

    let new_name = new_name.trim().trim_end_matches(".md");
    if new_name.is_empty() {
        return Err("name cannot be empty".to_string());
    }

    let new_path = old_path.with_file_name(format!("{}.md", new_name));
    if new_path.exists() && new_path != old_path {
        return Err(format!("a note named '{}' already exists", new_name));
    }

    let escaped = regex::escape(old_stem);
    let pattern = format!(r"(?i)\[\[{}(?:\.md)?(\|[^\]]+)?\]\]", escaped);
    let re = Regex::new(&pattern).map_err(|e| e.to_string())?;

    for entry in WalkDir::new(vault)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.') && !name.ends_with("~")
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let rewritten = re.replace_all(&content, |caps: &regex::Captures| {
            match caps.get(1) {
                Some(display) => format!("[[{}{}]]", new_name, display.as_str()),
                None => format!("[[{}]]", new_name),
            }
        });
        if rewritten != content {
            std::fs::write(path, rewritten.as_ref()).map_err(|e| e.to_string())?;
        }
    }

    std::fs::rename(&old_path, &new_path).map_err(|e| e.to_string())?;
    indexer::build_index(vault).map_err(|e| e.to_string())?;

    println!("{}", new_path.display());
    Ok(())
}

fn cmd_search(vault: &str, query: &str, json: bool) -> Result<(), String> {
    use crate::SearchResult;
    use walkdir::WalkDir;

    let mut results: Vec<SearchResult> = vec![];
    let query_lower = query.to_lowercase();

    for entry in WalkDir::new(vault)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.') && !name.ends_with("~") && name != "scratch"
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !path.is_file() || ext != "md" {
            continue;
        }
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for (line_num, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(&query_lower) {
                results.push(SearchResult {
                    path: path.to_string_lossy().to_string(),
                    name: path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string(),
                    line: line_num + 1,
                    content: line.to_string(),
                });
                if results.len() >= 100 {
                    break;
                }
            }
        }
        if results.len() >= 100 {
            break;
        }
    }

    if json {
        print_json(&results);
    } else {
        for r in &results {
            println!("{}:{}  {}", r.path, r.line, r.content);
        }
    }
    Ok(())
}

async fn cmd_semantic_search(
    vault: &str,
    query: &str,
    json: bool,
) -> Result<(), String> {
    let settings = settings::get_settings_cli();
    let results =
        semantic::semantic_search_with_settings(vault, query, 20, &settings).await?;

    if json {
        print_json(&results);
    } else {
        for r in &results {
            println!("{:.4}  {}:{}  {}", r.score, r.path, r.line, r.content);
        }
    }
    Ok(())
}

fn cmd_backlinks(vault: &str, note: &str, json: bool) -> Result<(), String> {
    let backlinks = indexer::get_backlinks(vault, note).map_err(|e| e.to_string())?;

    if json {
        print_json(&backlinks);
    } else if backlinks.is_empty() {
        println!("no backlinks found for '{}'", note);
    } else {
        for bl in &backlinks {
            println!("{}  {}", bl.note_name, bl.note_path);
        }
    }
    Ok(())
}

fn cmd_graph(vault: &str, json: bool) -> Result<(), String> {
    let graph = indexer::get_graph(vault).map_err(|e| e.to_string())?;

    if json {
        print_json(&graph);
    } else {
        println!("nodes: {}", graph.nodes.len());
        println!("edges: {}", graph.edges.len());
        for node in &graph.nodes {
            let kind_marker = if node.kind == "tag" { "#" } else { " " };
            println!("  {}{} (links: {})", kind_marker, node.id, node.link_count);
        }
    }
    Ok(())
}

fn cmd_tags(vault: &str, json: bool) -> Result<(), String> {
    let tags = indexer::get_tags(vault).map_err(|e| e.to_string())?;

    if json {
        print_json(&tags);
    } else {
        for t in &tags {
            println!("{} ({} notes)", t.tag, t.count);
        }
    }
    Ok(())
}

fn cmd_find(vault: &str, name: &str, json: bool) -> Result<(), String> {
    use walkdir::WalkDir;

    let target_lower = name.to_lowercase();
    for entry in WalkDir::new(vault)
        .into_iter()
        .filter_entry(|e| {
            let n = e.file_name().to_string_lossy();
            !n.starts_with('.') && n != "scratch"
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !path.is_file() || ext != "md" {
            continue;
        }
        let file_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        if file_name == target_lower {
            let path_str = path.to_string_lossy().to_string();
            if json {
                println!("{{\"path\":\"{}\"}}", path_str);
            } else {
                println!("{}", path_str);
            }
            return Ok(());
        }
    }

    if json {
        println!("null");
    } else {
        println!("not found");
    }
    Ok(())
}

async fn cmd_check(json: bool) -> Result<(), String> {
    let settings = settings::get_settings_cli();
    let health = semantic::test_ollama_with_settings(&settings).await?;

    let nvim_path = {
        let candidates: &[&str] = &[
            "nvim", "/opt/homebrew/bin/nvim", "/usr/local/bin/nvim",
            "/usr/bin/nvim", "vim", "/opt/homebrew/bin/vim",
            "/usr/local/bin/vim",
        ];
        candidates
            .iter()
            .find(|&c| {
                let p = Path::new(c);
                if p.is_absolute() { p.exists() } else { which::which(c).is_ok() }
            })
            .map(|c| c.to_string())
    };

    let neovim_installed = nvim_path.is_some();

    #[derive(Serialize)]
    struct Deps {
        neovim_installed: bool,
        neovim_path: Option<String>,
        ollama_reachable: bool,
        ollama_models_available: bool,
        ollama_message: String,
    }

    let deps = Deps {
        neovim_installed,
        neovim_path: nvim_path,
        ollama_reachable: health.reachable,
        ollama_models_available: health.model_available,
        ollama_message: health.message.clone(),
    };

    if json {
        print_json(&deps);
    } else {
        println!("neovim:    {}", if neovim_installed { "✓" } else { "✗" });
        if let Some(ref p) = deps.neovim_path {
            println!("  path:    {}", p);
        }
        println!("ollama:    {}", if health.reachable { "✓" } else { "✗" });
        println!("  {}", health.message);
    }

    Ok(())
}

async fn cmd_rebuild_index(vault: &str) -> Result<(), String> {
    let settings = settings::get_settings_cli();
    eprintln!("building semantic index for {}", vault);
    semantic::semantic_index_rebuild_with_settings(vault, &settings).await?;
    eprintln!("done");
    Ok(())
}

fn cmd_add_to_path() -> Result<(), String> {
    let bin = std::env::current_exe().map_err(|e| e.to_string())?;

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let candidates: &[PathBuf] = &[
        PathBuf::from("/usr/local/bin/synaptic"),
        PathBuf::from(&home).join(".local/bin/synaptic"),
    ];

    for target in candidates {
        if target.exists() {
            if target.is_symlink() {
                println!("already installed: {}", target.display());
                return Ok(());
            }
            eprintln!("warning: {} exists but is not a symlink, skipping", target.display());
            continue;
        }

        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        match std::os::unix::fs::symlink(&bin, target) {
            Ok(()) => {
                println!("symlinked {} -> {}", target.display(), bin.display());
                return Ok(());
            }
            Err(e) => {
                eprintln!("could not create {}: {}", target.display(), e);
                continue;
            }
        }
    }

    Err("could not symlink — try with sudo, or add ~/.local/bin to your PATH".to_string())
}

fn cmd_remove_from_path() -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let candidates: &[PathBuf] = &[
        PathBuf::from("/usr/local/bin/synaptic"),
        PathBuf::from(&home).join(".local/bin/synaptic"),
    ];

    for target in candidates {
        if target.is_symlink() {
            std::fs::remove_file(target).map_err(|e| e.to_string())?;
            println!("removed {}", target.display());
            return Ok(());
        }
    }

    println!("no symlink found");
    Ok(())
}

// ─── Entry Point ─────────────────────────────────────────────────────────────

pub async fn run() -> Result<(), String> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Open { path } => cmd_open(path, cli.json),

        Commands::ListRecent => cmd_list_recent(cli.json),

        Commands::Tree { path } => {
            let vault = require_vault(&cli)?;
            cmd_tree(&vault, path.as_deref(), cli.json)
        }

        Commands::Read { path } => {
            let vault = resolve_vault(&cli).ok();
            cmd_read(vault.as_deref(), path, cli.json)
        }

        Commands::Create { path } => {
            let vault = require_vault(&cli)?;
            cmd_create(&vault, path)
        }

        Commands::Write { path, content } => {
            let vault = require_vault(&cli)?;
            cmd_write(&vault, path, content.as_deref())
        }

        Commands::Delete { path } => {
            let vault = require_vault(&cli)?;
            cmd_delete(&vault, path)
        }

        Commands::Rename { old, new } => {
            let vault = require_vault(&cli)?;
            cmd_rename(&vault, old, new)
        }

        Commands::Search { query } => {
            let vault = require_vault(&cli)?;
            cmd_search(&vault, query, cli.json)
        }

        Commands::SemanticSearch { query } => {
            let vault = require_vault(&cli)?;
            cmd_semantic_search(&vault, query, cli.json).await
        }

        Commands::Backlinks { note } => {
            let vault = require_vault(&cli)?;
            cmd_backlinks(&vault, note, cli.json)
        }

        Commands::Graph => {
            let vault = require_vault(&cli)?;
            cmd_graph(&vault, cli.json)
        }

        Commands::Tags => {
            let vault = require_vault(&cli)?;
            cmd_tags(&vault, cli.json)
        }

        Commands::Find { name } => {
            let vault = require_vault(&cli)?;
            cmd_find(&vault, name, cli.json)
        }

        Commands::Check => cmd_check(cli.json).await,

        Commands::RebuildIndex => {
            let vault = require_vault(&cli)?;
            cmd_rebuild_index(&vault).await
        }

        Commands::AddToPath => cmd_add_to_path(),

        Commands::RemoveFromPath => cmd_remove_from_path(),
    }
}
