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

fn default_kind() -> String {
    "note".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GraphNode {
    pub id: String,
    pub path: String,
    pub link_count: usize,
    #[serde(default = "default_kind")]
    pub kind: String,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TagNote {
    pub name: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TagEntry {
    pub tag: String,
    pub count: usize,
    pub notes: Vec<TagNote>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Index {
    pub links: HashMap<String, Vec<String>>,
    pub backlinks: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub tags: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub tag_notes: HashMap<String, Vec<String>>,
}

fn parse_tags(content: &str) -> Vec<String> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return vec![];
    }
    let after_open = trimmed[3..].trim_start_matches('\r').trim_start_matches('\n');
    let fm_end = match after_open.find("\n---") {
        Some(pos) => pos,
        None => return vec![],
    };
    let frontmatter = &after_open[..fm_end];

    let mut tags = vec![];
    let mut in_tags_list = false;

    for line in frontmatter.lines() {
        let trimmed_line = line.trim();
        if let Some(after_colon) = trimmed_line.strip_prefix("tags:") {
            in_tags_list = false;
            let val = after_colon.trim();
            if val.starts_with('[') {
                let inner = val.trim_start_matches('[').trim_end_matches(']');
                for t in inner.split(',') {
                    let tag = t.trim().trim_matches('"').trim_matches('\'').to_lowercase();
                    if !tag.is_empty() {
                        tags.push(tag);
                    }
                }
            } else if val.is_empty() {
                in_tags_list = true;
            } else {
                for t in val.split(',') {
                    let tag = t.trim().trim_matches('"').trim_matches('\'').to_lowercase();
                    if !tag.is_empty() {
                        tags.push(tag);
                    }
                }
            }
        } else if in_tags_list {
            if let Some(rest) = trimmed_line.strip_prefix('-') {
                let tag = rest.trim().trim_matches('"').trim_matches('\'').to_lowercase();
                if !tag.is_empty() {
                    tags.push(tag);
                }
            } else if !trimmed_line.is_empty() {
                in_tags_list = false;
            }
        }
    }

    tags
}

fn walk_vault(system_path: &str) -> impl Iterator<Item = walkdir::DirEntry> {
    WalkDir::new(system_path)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.') && name != "scratch"
        })
        .filter_map(|e| e.ok())
}

fn build_path_map(system_path: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for entry in walk_vault(system_path) {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        map.insert(stem, path.to_string_lossy().to_string());
    }
    map
}

pub fn build_index(system_path: &str) -> Result<(), std::io::Error> {
    let mut links: HashMap<String, Vec<String>> = HashMap::new();
    let mut note_tags: HashMap<String, Vec<String>> = HashMap::new();
    let wiki_re = Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]*)?\]\]").unwrap();

    for entry in walk_vault(system_path) {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();

        let mut targets = HashSet::new();
        for cap in wiki_re.captures_iter(&content) {
            let mut target = cap[1].trim().to_lowercase();
            if target.ends_with(".md") {
                target.truncate(target.len() - 3);
            }
            targets.insert(target);
        }
        links.insert(name.clone(), targets.into_iter().collect());

        let tags = parse_tags(&content);
        if !tags.is_empty() {
            note_tags.insert(name, tags);
        }
    }

    let mut backlinks: HashMap<String, Vec<String>> = HashMap::new();
    for (source, targets) in &links {
        for target in targets {
            backlinks.entry(target.clone()).or_default().push(source.clone());
        }
    }

    let mut tag_notes: HashMap<String, Vec<String>> = HashMap::new();
    for (note, tags) in &note_tags {
        for tag in tags {
            tag_notes.entry(tag.clone()).or_default().push(note.clone());
        }
    }

    let index = Index { links, backlinks, tags: note_tags, tag_notes };
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

    let path_map = build_path_map(system_path);
    let result = backlink_names.into_iter().filter_map(|bl_name| {
        path_map.get(&bl_name).map(|path| BacklinkInfo {
            note_name: bl_name.clone(),
            note_path: path.clone(),
        })
    }).collect();

    Ok(result)
}

pub fn get_tags(system_path: &str) -> Result<Vec<TagEntry>, std::io::Error> {
    build_index(system_path)?;
    let index = load_index(system_path)?;
    let path_map = build_path_map(system_path);

    let mut entries: Vec<TagEntry> = index.tag_notes.iter().map(|(tag, note_names)| {
        let notes: Vec<TagNote> = note_names.iter().filter_map(|name| {
            path_map.get(name).map(|path| TagNote {
                name: name.clone(),
                path: path.clone(),
            })
        }).collect();
        let count = notes.len();
        TagEntry { tag: tag.clone(), count, notes }
    }).collect();

    entries.sort_by(|a, b| a.tag.cmp(&b.tag));
    Ok(entries)
}

pub fn get_graph(system_path: &str) -> Result<GraphData, std::io::Error> {
    build_index(system_path)?;
    let index = load_index(system_path)?;

    let path_map = build_path_map(system_path);

    let mut note_ids: HashSet<String> = HashSet::new();
    let mut edges: Vec<GraphEdge> = vec![];

    for (source, targets) in &index.links {
        note_ids.insert(source.clone());
        for target in targets {
            note_ids.insert(target.clone());
            edges.push(GraphEdge {
                source: source.clone(),
                target: target.clone(),
            });
        }
    }
    for (target, sources) in &index.backlinks {
        note_ids.insert(target.clone());
        for source in sources {
            note_ids.insert(source.clone());
        }
    }

    // Include notes that have tags but no wiki-links
    for note_names in index.tag_notes.values() {
        for name in note_names {
            if path_map.contains_key(name) {
                note_ids.insert(name.clone());
            }
        }
    }

    let mut nodes: Vec<GraphNode> = note_ids.iter().map(|id| {
        let link_count = index.links.get(id).map(|v| v.len()).unwrap_or(0)
            + index.backlinks.get(id).map(|v| v.len()).unwrap_or(0);
        GraphNode {
            id: id.clone(),
            path: path_map.get(id).cloned().unwrap_or_default(),
            link_count,
            kind: "note".to_string(),
        }
    }).collect();

    // Tag nodes and edges
    for (tag, note_names) in &index.tag_notes {
        let tag_id = format!("#{}", tag);
        let note_count = note_names.iter().filter(|n| note_ids.contains(*n)).count();
        if note_count == 0 {
            continue;
        }
        nodes.push(GraphNode {
            id: tag_id.clone(),
            path: String::new(),
            link_count: note_count,
            kind: "tag".to_string(),
        });
        for note_name in note_names {
            if note_ids.contains(note_name) {
                edges.push(GraphEdge {
                    source: note_name.clone(),
                    target: tag_id.clone(),
                });
            }
        }
    }

    Ok(GraphData { nodes, edges })
}
