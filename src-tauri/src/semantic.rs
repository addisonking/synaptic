use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::settings::get_settings;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmbeddingChunk {
    pub path: String,
    pub name: String,
    pub line: usize,
    pub content: String,
    pub vector: Vec<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SemanticIndex {
    pub model: String,
    pub built_at: i64,
    pub chunks: Vec<EmbeddingChunk>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SemanticResult {
    pub path: String,
    pub name: String,
    pub line: usize,
    pub content: String,
    pub score: f64,
}

fn semantic_index_path(system_path: &str) -> std::path::PathBuf {
    Path::new(system_path).join(".synaptic").join("semantic_index.json")
}

fn chunk_content(content: &str) -> Vec<(usize, String)> {
    let max_len = 1000;
    let lines: Vec<&str> = content.lines().collect();
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut start_line = 1;

    for (i, line) in lines.iter().enumerate() {
        if current.len() + line.len() > max_len && !current.is_empty() {
            chunks.push((start_line, current.trim().to_string()));
            // Overlap: include last few lines of previous chunk
            let overlap_lines: Vec<&str> = current.lines().rev().take(3).collect();
            current = overlap_lines.into_iter().rev().collect::<Vec<_>>().join("\n");
            current.push('\n');
            current.push_str(line);
            current.push('\n');
            start_line = i.saturating_sub(2) + 1;
        } else {
            if current.is_empty() {
                start_line = i + 1;
            }
            current.push_str(line);
            current.push('\n');
        }
    }

    if !current.trim().is_empty() {
        chunks.push((start_line, current.trim().to_string()));
    }

    chunks
}

#[derive(Serialize)]
struct OllamaEmbedRequest<'a> {
    model: &'a str,
    prompt: &'a str,
}

#[derive(Deserialize)]
struct OllamaEmbedResponse {
    embedding: Vec<f32>,
}

async fn get_embedding(
    client: &reqwest::Client,
    url: &str,
    model: &str,
    text: &str,
) -> Result<Vec<f32>, String> {
    let req = OllamaEmbedRequest { model, prompt: text };
    let res = client
        .post(format!("{}/api/embeddings", url.trim_end_matches('/')))
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

    let body: OllamaEmbedResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

    if body.embedding.is_empty() {
        return Err("Ollama returned an empty embedding. Is the model loaded correctly?".to_string());
    }

    Ok(body.embedding)
}

fn normalize(v: &mut [f32]) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    dot as f64
}

pub async fn semantic_index_rebuild(
    system_path: &str,
    app: &tauri::AppHandle,
) -> Result<(), String> {
    let settings = get_settings(app).map_err(|e| e.to_string())?;
    let url = settings
        .ollama_url
        .unwrap_or_else(|| "http://localhost:11434".to_string());
    let model = settings
        .ollama_model
        .unwrap_or_else(|| "nomic-embed-text".to_string());

    let client = reqwest::Client::new();
    let mut chunks: Vec<EmbeddingChunk> = Vec::new();

    for entry in WalkDir::new(system_path)
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
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let file_chunks = chunk_content(&content);
        for (line, text) in file_chunks {
            if text.len() < 20 {
                continue;
            }
            match get_embedding(&client, &url, &model, &text).await {
                Ok(mut vector) => {
                    normalize(&mut vector);
                    chunks.push(EmbeddingChunk {
                        path: path.to_string_lossy().to_string(),
                        name: name.clone(),
                        line,
                        content: text,
                        vector,
                    });
                }
                Err(e) => {
                    eprintln!("Embedding error for {}: {}", path.display(), e);
                }
            }
        }
    }

    let index = SemanticIndex {
        model: model.clone(),
        built_at: chrono::Utc::now().timestamp(),
        chunks,
    };

    let index_path = semantic_index_path(system_path);
    if let Some(parent) = index_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        &index_path,
        serde_json::to_string_pretty(&index).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn semantic_search(
    system_path: &str,
    query: &str,
    top_k: usize,
    app: &tauri::AppHandle,
) -> Result<Vec<SemanticResult>, String> {
    let index_path = semantic_index_path(system_path);
    if !index_path.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&index_path).map_err(|e| e.to_string())?;
    let index: SemanticIndex = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let settings = get_settings(app).map_err(|e| e.to_string())?;
    let url = settings
        .ollama_url
        .unwrap_or_else(|| "http://localhost:11434".to_string());
    let model = settings
        .ollama_model
        .unwrap_or_else(|| "nomic-embed-text".to_string());

    let client = reqwest::Client::new();
    let mut query_vector = get_embedding(&client, &url, &model, query).await?;
    normalize(&mut query_vector);

    let mut results: Vec<SemanticResult> = index
        .chunks
        .into_iter()
        .map(|chunk| {
            let score = cosine_similarity(&query_vector, &chunk.vector);
            SemanticResult {
                path: chunk.path,
                name: chunk.name,
                line: chunk.line,
                content: chunk.content,
                score,
            }
        })
        .collect();

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(top_k);

    Ok(results)
}

pub fn semantic_index_model(system_path: &str) -> Option<String> {
    let index_path = semantic_index_path(system_path);
    if !index_path.exists() {
        return None;
    }
    let content = fs::read_to_string(&index_path).ok()?;
    let index: SemanticIndex = serde_json::from_str(&content).ok()?;
    Some(index.model)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OllamaHealth {
    pub reachable: bool,
    pub model_available: bool,
    pub message: String,
}

pub async fn test_ollama_connection(
    app: &tauri::AppHandle,
) -> Result<OllamaHealth, String> {
    let settings = get_settings(app).map_err(|e| e.to_string())?;
    let url = settings
        .ollama_url
        .unwrap_or_else(|| "http://localhost:11434".to_string());
    let model = settings
        .ollama_model
        .unwrap_or_else(|| "nomic-embed-text".to_string());

    let client = reqwest::Client::new();

    // 1. Check if Ollama is reachable
    let tags_res = client
        .get(format!("{}/api/tags", url.trim_end_matches('/')))
        .send()
        .await;

    match tags_res {
        Ok(res) => {
            if !res.status().is_success() {
                return Ok(OllamaHealth {
                    reachable: false,
                    model_available: false,
                    message: format!("Ollama returned HTTP {}", res.status()),
                });
            }

            // 2. Check if model exists
            #[derive(Deserialize)]
            struct TagEntry {
                name: String,
            }
            #[derive(Deserialize)]
            struct TagsResponse {
                models: Vec<TagEntry>,
            }

            let tags: TagsResponse = res.json().await.map_err(|e| e.to_string())?;
            let model_found = tags.models.iter().any(|m| {
                m.name == model || m.name == format!("{}:latest", model) || m.name.starts_with(&format!("{}:", model))
            });

            if model_found {
                Ok(OllamaHealth {
                    reachable: true,
                    model_available: true,
                    message: format!("Connected. Model '{}' is available.", model),
                })
            } else {
                Ok(OllamaHealth {
                    reachable: true,
                    model_available: false,
                    message: format!(
                        "Ollama is reachable, but model '{}' was not found. Run: ollama pull {}",
                        model, model
                    ),
                })
            }
        }
        Err(e) => {
            let msg = if e.is_connect() || e.is_request() {
                format!("Cannot connect to Ollama at {}. Is it running?", url)
            } else {
                format!("Ollama request failed: {}", e)
            };
            Ok(OllamaHealth {
                reachable: false,
                model_available: false,
                message: msg,
            })
        }
    }
}
