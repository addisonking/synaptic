use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BacklinkInfo {
    pub note_name: String,
    pub note_path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GraphNode {
    pub id: String,
    pub path: String,
    pub link_count: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Index {
    pub links: HashMap<String, Vec<String>>,
    pub backlinks: HashMap<String, Vec<String>>,
}

pub fn build_index(system_path: &str) -> Result<(), std::io::Error> {
    let mut links: HashMap<String, Vec<String>> = HashMap::new();

    let wiki_re1 = Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]*)?\]\]").unwrap();

    for entry in WalkDir::new(system_path)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.') && name != "scratch"
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
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();

        let mut targets = HashSet::new();
        for cap in wiki_re1.captures_iter(&content) {
            let mut target = cap[1].trim().to_lowercase();
            // Strip .md extension so [[Note.md]] resolves to the same node as the file
            if target.ends_with(".md") {
                target.truncate(target.len() - 3);
            }
            targets.insert(target);
        }
        links.insert(name.to_lowercase(), targets.into_iter().collect());
    }

    // Build backlinks
    let mut backlinks: HashMap<String, Vec<String>> = HashMap::new();
    for (source, targets) in &links {
        for target in targets {
            backlinks.entry(target.clone()).or_default().push(source.clone());
        }
    }

    let index = Index { links, backlinks };
    let index_path = Path::new(system_path).join(".synaptic").join("index.json");
    fs::write(&index_path, serde_json::to_string_pretty(&index)?)?;
    Ok(())
}

pub fn load_index(system_path: &str) -> Result<Index, std::io::Error> {
    let index_path = Path::new(system_path).join(".synaptic").join("index.json");
    if !index_path.exists() {
        build_index(system_path)?;
    }
    let content = fs::read_to_string(&index_path)?;
    Ok(serde_json::from_str(&content).unwrap_or_default())
}

pub fn get_backlinks(system_path: &str, note_name: &str) -> Result<Vec<BacklinkInfo>, std::io::Error> {
    build_index(system_path)?;
    let index = load_index(system_path)?;
    let name_lower = note_name.to_lowercase();
    let backlink_names = index.backlinks.get(&name_lower).cloned().unwrap_or_default();

    let mut result = vec![];
    for bl_name in backlink_names {
        for entry in WalkDir::new(system_path)
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
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            if stem == bl_name {
                result.push(BacklinkInfo {
                    note_name: path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string(),
                    note_path: path.to_string_lossy().to_string(),
                });
                break;
            }
        }
    }
    Ok(result)
}

pub fn get_graph(system_path: &str) -> Result<GraphData, std::io::Error> {
    build_index(system_path)?;
    let index = load_index(system_path)?;
    let mut nodes = vec![];
    let mut edges = vec![];
    let mut node_ids = HashSet::new();

    for (source, targets) in &index.links {
        node_ids.insert(source.clone());
        for target in targets {
            node_ids.insert(target.clone());
            edges.push(GraphEdge {
                source: source.clone(),
                target: target.clone(),
            });
        }
    }

    // Add nodes from backlinks too (orphans)
    for (target, sources) in &index.backlinks {
        node_ids.insert(target.clone());
        for source in sources {
            node_ids.insert(source.clone());
        }
    }

    // Find paths for all nodes
    let mut path_map: HashMap<String, String> = HashMap::new();
    for entry in WalkDir::new(system_path)
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
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        path_map.insert(stem, path.to_string_lossy().to_string());
    }

    for id in node_ids {
        let link_count = index.links.get(&id).map(|v| v.len()).unwrap_or(0)
            + index.backlinks.get(&id).map(|v| v.len()).unwrap_or(0);
        nodes.push(GraphNode {
            id: id.clone(),
            path: path_map.get(&id).cloned().unwrap_or_default(),
            link_count,
        });
    }

    Ok(GraphData { nodes, edges })
}
