use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::indexer::build_index;
use crate::semantic;
use crate::semantic::http_client;
use crate::settings::get_settings;
use tauri::ipc::Channel;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GhostSource {
    pub note_path: String,
    pub note_name: String,
    pub line: usize,
    pub context: String,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GhostLink {
    pub target: String,
    pub sources: Vec<GhostSource>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GhostNotePreview {
    pub target: String,
    pub content: String,
}

fn notes_dir(vault_path: &str) -> std::path::PathBuf {
    Path::new(vault_path).join("notes")
}

fn extract_context(content: &str, link_line: usize, window: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let start = link_line.saturating_sub(window + 1);
    let end = (link_line + window).min(lines.len());
    lines[start..end].join("\n")
}

fn extract_frontmatter_tags(content: &str) -> Vec<String> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return vec![];
    }
    let after_open = &trimmed[3..];
    let after_open = after_open
        .trim_start_matches('\r')
        .trim_start_matches('\n');
    let Some(end_pos) = after_open.find("\n---") else {
        return vec![];
    };
    let fm = &after_open[..end_pos];

    let mut tags: Vec<String> = Vec::new();
    let mut in_array = false;
    let mut base_indent: usize = 0;

    for line in fm.lines() {
        let line_trimmed = line.trim();
        if line_trimmed.is_empty() {
            continue;
        }

        let indent = line.chars().take_while(|c| c.is_whitespace()).count();

        if in_array {
            if indent <= base_indent && line_trimmed.starts_with(|c: char| c.is_alphanumeric()) {
                in_array = false;
            } else if line_trimmed.starts_with('-') {
                let val = line_trimmed[1..].trim();
                if !val.is_empty() {
                    tags.push(val.to_string());
                }
                continue;
            }
        }

        if !in_array {
            if let Some(colon_pos) = line_trimmed.find(':') {
                let key = line_trimmed[..colon_pos].trim();
                let val = line_trimmed[colon_pos + 1..].trim();
                if key == "tags" || key == "tag" {
                    if val.is_empty() {
                        in_array = true;
                        base_indent = indent;
                    } else if val.starts_with('[') && val.ends_with(']') {
                        for item in val[1..val.len() - 1].split(',') {
                            let item = item.trim();
                            if !item.is_empty() {
                                tags.push(item.to_string());
                            }
                        }
                    } else {
                        for item in val.split(',') {
                            let item = item.trim();
                            if !item.is_empty() {
                                tags.push(item.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    tags
}

#[tauri::command]
pub fn scan_ghost_links_cmd(system_path: String) -> Result<Vec<GhostLink>, String> {
    let wiki_re = Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]*)?\]\]").unwrap();
    let mut raw: HashMap<String, Vec<GhostSource>> = HashMap::new();

    // Build a set of existing note names once for O(1) lookups
    let mut existing_names = HashSet::new();
    for entry in WalkDir::new(&system_path)
        .into_iter()
        .filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()).unwrap_or("") != "md" {
            continue;
        }
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        if !name.is_empty() {
            existing_names.insert(name);
        }
    }

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
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let note_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        let note_path = path.to_string_lossy().to_string();
        let note_tags = extract_frontmatter_tags(&content);

        for (line_num, line) in content.lines().enumerate() {
            for cap in wiki_re.captures_iter(line) {
                let mut target = cap[1].trim().to_string();
                // Strip .md extension so [[Note.md]] resolves the same way
                if target.ends_with(".md") {
                    target.truncate(target.len() - 3);
                }
                if target.is_empty() {
                    continue;
                }
                if existing_names.contains(&target.to_lowercase()) {
                    continue;
                }
                let ctx = extract_context(&content, line_num + 1, 3);
                raw.entry(target.clone()).or_default().push(GhostSource {
                    note_path: note_path.clone(),
                    note_name: note_name.clone(),
                    line: line_num + 1,
                    context: ctx,
                    tags: note_tags.clone(),
                });
            }
        }
    }

    let mut result: Vec<GhostLink> = raw
        .into_iter()
        .map(|(target, sources)| GhostLink { target, sources })
        .collect();
    result.sort_by(|a, b| b.sources.len().cmp(&a.sources.len()));
    Ok(result)
}

#[derive(Serialize)]
struct OllamaGenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

#[derive(Deserialize)]
struct OllamaStreamResponse {
    response: String,
    done: bool,
}

#[derive(Serialize, Clone)]
pub struct GhostStreamChunk {
    kind: String,
    data: String,
}

async fn ollama_generate(
    client: &reqwest::Client,
    url: &str,
    model: &str,
    prompt: &str,
) -> Result<String, String> {
    let req = OllamaGenerateRequest {
        model,
        prompt,
        stream: false,
    };
    let res = client
        .post(format!("{}/api/generate", url.trim_end_matches('/')))
        .json(&req)
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() || e.is_request() {
                format!("Cannot connect to Ollama at {}. Is it running?", url)
            } else {
                format!("Ollama request failed: {}", e)
            }
        })?;

    if !res.status().is_success() {
        let status = res.status();
        let body_text = res.text().await.unwrap_or_default();
        if body_text.to_lowercase().contains("model") && body_text.to_lowercase().contains("not found") {
            return Err(format!(
                "Model '{}' not found. Run: ollama pull {}",
                model, model
            ));
        }
        return Err(format!(
            "Ollama returned HTTP {}: {}",
            status, body_text
        ));
    }

    let body: OllamaGenerateResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;
    Ok(body.response.trim().to_string())
}

async fn ollama_generate_stream(
    client: &reqwest::Client,
    url: &str,
    model: &str,
    prompt: &str,
    on_chunk: &Channel<GhostStreamChunk>,
) -> Result<(), String> {
    let req = OllamaGenerateRequest {
        model,
        prompt,
        stream: true,
    };
    let mut res = client
        .post(format!("{}/api/generate", url.trim_end_matches('/')))
        .json(&req)
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() || e.is_request() {
                format!("Cannot connect to Ollama at {}. Is it running?", url)
            } else {
                format!("Ollama request failed: {}", e)
            }
        })?;

    if !res.status().is_success() {
        let status = res.status();
        let body_text = res.text().await.unwrap_or_default();
        if body_text.to_lowercase().contains("model") && body_text.to_lowercase().contains("not found") {
            return Err(format!(
                "Model '{}' not found. Run: ollama pull {}",
                model, model
            ));
        }
        return Err(format!(
            "Ollama returned HTTP {}: {}",
            status, body_text
        ));
    }

    let mut buffer = String::new();

    while let Some(chunk) = res.chunk().await.map_err(|e| format!("Stream error: {}", e))? {
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(pos) = buffer.find('\n') {
            let line = buffer.drain(..=pos).collect::<String>();
            let line = line.trim();
            if line.is_empty() { continue; }

            if let Ok(json) = serde_json::from_str::<OllamaStreamResponse>(line) {
                on_chunk.send(GhostStreamChunk {
                    kind: "chunk".to_string(),
                    data: json.response,
                }).map_err(|e| e.to_string())?;

                if json.done {
                    on_chunk.send(GhostStreamChunk {
                        kind: "done".to_string(),
                        data: String::new(),
                    }).map_err(|e| e.to_string())?;
                    return Ok(());
                }
            }
        }
    }

    on_chunk.send(GhostStreamChunk {
        kind: "done".to_string(),
        data: String::new(),
    }).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn preview_ghost_note_cmd(
    system_path: String,
    target: String,
    sources: Vec<GhostSource>,
    app: tauri::AppHandle,
) -> Result<GhostNotePreview, String> {
    let settings = get_settings(&app).map_err(|e| e.to_string())?;
    let url = settings
        .ollama_url
        .unwrap_or_else(|| "http://localhost:11434".to_string());
    let model = settings
        .ghost_model
        .or(settings.ollama_model)
        .unwrap_or_else(|| "gemma4:26b".to_string());

    // Build source context block
    let mut source_contexts = String::new();
    let mut all_tags: Vec<String> = Vec::new();
    for (i, src) in sources.iter().enumerate() {
        source_contexts.push_str(&format!(
            "\n--- Source {}: {} (line {}) ---\n{}\n",
            i + 1,
            src.note_name,
            src.line,
            src.context
        ));
        for tag in &src.tags {
            if !all_tags.contains(tag) {
                all_tags.push(tag.clone());
            }
        }
    }

    let tags_section = if all_tags.is_empty() {
        String::new()
    } else {
        format!(
            "\nThe source notes reference this topic with the following frontmatter tags: {}. \
If it makes sense for the topic \"{target}\", include a YAML frontmatter block at the very top of the note with relevant tags:\n\n---\ntags: [tag1, tag2]\n---\n",
            all_tags.join(", ")
        )
    };

    // Get semantic matches for related notes
    let mut related_notes = String::new();
    match semantic::semantic_search(&system_path, &target, 3, &app).await {
        Ok(results) => {
            for (i, r) in results.iter().enumerate() {
                related_notes.push_str(&format!(
                    "\n--- Related {}: {} ---\n{}\n",
                    i + 1,
                    r.name,
                    r.content
                ));
            }
        }
        Err(e) => {
            eprintln!("Semantic search failed for ghost preview: {}", e);
        }
    }

    let prompt = format!(
        r#"You are writing a markdown note titled "{target}" for a personal knowledge base.

This note is referenced from existing notes in these contexts:
{source_contexts}
{tags_section}
{related_notes_section}

Write a concise, information-dense markdown note. Match the writing style and density of the source context. Start with `# {target}`. Include natural backlinks to related notes using `[[Note Name]]` syntax where relevant. Do not include meta-commentary, filler, or summaries about what you are doing. Every sentence should carry information."#,
        target = target,
        source_contexts = source_contexts,
        tags_section = tags_section,
        related_notes_section = if related_notes.is_empty() {
            "".to_string()
        } else {
            format!("Related notes in this knowledge base:\n{}", related_notes)
        },
    );

    let client = http_client();
    let content = ollama_generate(&client, &url, &model, &prompt).await?;

    // Ensure it starts with a heading
    let content = if !content.starts_with("# ") {
        format!("# {}\n\n{}", target, content)
    } else {
        content
    };

    Ok(GhostNotePreview { target, content })
}

#[tauri::command]
pub async fn preview_ghost_note_stream_cmd(
    system_path: String,
    target: String,
    sources: Vec<GhostSource>,
    on_chunk: Channel<GhostStreamChunk>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let settings = get_settings(&app).map_err(|e| e.to_string())?;
    let url = settings
        .ollama_url
        .unwrap_or_else(|| "http://localhost:11434".to_string());
    let model = settings
        .ghost_model
        .or(settings.ollama_model)
        .unwrap_or_else(|| "gemma4:26b".to_string());

    on_chunk.send(GhostStreamChunk {
        kind: "status".to_string(),
        data: "Building source context...".to_string(),
    }).map_err(|e| e.to_string())?;

    let mut source_contexts = String::new();
    let mut all_tags: Vec<String> = Vec::new();
    for (i, src) in sources.iter().enumerate() {
        source_contexts.push_str(&format!(
            "\n--- Source {}: {} (line {}) ---\n{}\n",
            i + 1,
            src.note_name,
            src.line,
            src.context
        ));
        for tag in &src.tags {
            if !all_tags.contains(tag) {
                all_tags.push(tag.clone());
            }
        }
    }

    let tags_section = if all_tags.is_empty() {
        String::new()
    } else {
        format!(
            "\nThe source notes reference this topic with the following frontmatter tags: {}. \
If it makes sense for the topic \"{target}\", include a YAML frontmatter block at the very top of the note with relevant tags:\n\n---\ntags: [tag1, tag2]\n---\n",
            all_tags.join(", ")
        )
    };

    on_chunk.send(GhostStreamChunk {
        kind: "status".to_string(),
        data: "Searching for related notes...".to_string(),
    }).map_err(|e| e.to_string())?;

    let mut related_notes = String::new();
    match semantic::semantic_search(&system_path, &target, 3, &app).await {
        Ok(results) => {
            for (i, r) in results.iter().enumerate() {
                related_notes.push_str(&format!(
                    "\n--- Related {}: {} ---\n{}\n",
                    i + 1,
                    r.name,
                    r.content
                ));
            }
        }
        Err(e) => {
            eprintln!("Semantic search failed for ghost preview: {}", e);
        }
    }

    let prompt = format!(
        r#"You are writing a markdown note titled "{target}" for a personal knowledge base.

This note is referenced from existing notes in these contexts:
{source_contexts}
{tags_section}
{related_notes_section}

Write a concise, information-dense markdown note. Match the writing style and density of the source context. Start with `# {target}`. Include natural backlinks to related notes using `[[Note Name]]` syntax where relevant. Do not include meta-commentary, filler, or summaries about what you are doing. Every sentence should carry information."#,
        target = target,
        source_contexts = source_contexts,
        tags_section = tags_section,
        related_notes_section = if related_notes.is_empty() {
            "".to_string()
        } else {
            format!("Related notes in this knowledge base:\n{}", related_notes)
        },
    );

    on_chunk.send(GhostStreamChunk {
        kind: "status".to_string(),
        data: "Generating note (this may take a moment)...".to_string(),
    }).map_err(|e| e.to_string())?;

    let client = http_client();
    ollama_generate_stream(&client, &url, &model, &prompt, &on_chunk).await?;

    Ok(())
}

#[tauri::command]
pub fn create_ghost_notes_cmd(
    system_path: String,
    notes: Vec<GhostNotePreview>,
) -> Result<(), String> {
    let notes_dir = notes_dir(&system_path);
    if !notes_dir.exists() {
        fs::create_dir_all(&notes_dir).map_err(|e| e.to_string())?;
    }

    for note in notes {
        let file_name = format!("{}.md", note.target);
        let path = notes_dir.join(&file_name);
        fs::write(&path, note.content).map_err(|e| e.to_string())?;
    }

    // Rebuild index so backlinks are available immediately
    build_index(&system_path).map_err(|e| e.to_string())?;

    Ok(())
}
