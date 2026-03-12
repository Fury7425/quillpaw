use chrono::Utc;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::models::{FileNode, NoteContent};

const FRONTMATTER_DELIM: &str = "---";

#[derive(Default, Clone)]
struct Frontmatter {
    title: Option<String>,
    created: Option<String>,
    modified: Option<String>,
    tags: Vec<String>,
}

/// Ensure the vault directory has required Quillpaw subfolders.
pub async fn ensure_vault_structure(vault_path: &str) -> Result<(), String> {
    let assets = [
        ".assets/images",
        ".assets/drawings",
        ".assets/audio",
        ".assets/files",
    ];
    let system = [".quillpaw/index", ".quillpaw/embeddings", ".quillpaw/models"];
    for folder in assets.iter().chain(system.iter()) {
        let path = Path::new(vault_path).join(folder);
        fs::create_dir_all(&path).await.map_err(|e| e.to_string())?;
    }
    let config_path = Path::new(vault_path).join(".quillpaw/config.json");
    if fs::metadata(&config_path).await.is_err() {
        fs::write(&config_path, "{}").await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Build a recursive file tree from a vault path.
pub async fn build_tree(path: &str) -> Result<Vec<FileNode>, String> {
    build_tree_inner(Path::new(path)).await
}

/// Read a note file and parse its frontmatter.
pub async fn read_note_file(path: &str) -> Result<NoteContent, String> {
    let content = fs::read_to_string(path).await.map_err(|e| e.to_string())?;
    let (meta, body) = parse_frontmatter(&content);
    let now = Utc::now().to_rfc3339();
    let title = meta
        .title
        .clone()
        .unwrap_or_else(|| title_from_path(Path::new(path)));
    Ok(NoteContent {
        path: path.to_string(),
        title,
        body,
        tags: meta.tags,
        created: meta.created.unwrap_or_else(|| now.clone()),
        modified: meta.modified.unwrap_or(now),
    })
}

/// Save a note body and refresh its frontmatter metadata.
pub async fn save_note_body(path: &str, body: &str) -> Result<(), String> {
    let existing = fs::read_to_string(path).await.unwrap_or_default();
    let (mut meta, _) = parse_frontmatter(&existing);
    let now = Utc::now().to_rfc3339();
    if meta.created.is_none() {
        meta.created = Some(now.clone());
    }
    meta.modified = Some(now);
    if meta.title.is_none() {
        meta.title = Some(title_from_path(Path::new(path)));
    }
    let frontmatter = build_frontmatter(&meta);
    let content = format!("{frontmatter}\n\n{body}");
    save_atomic(path, &content).await
}

/// Append a new section to a note body.
pub async fn append_note_section(path: &str, heading: &str, content: &str) -> Result<(), String> {
    let note = read_note_file(path).await?;
    let mut body = note.body.trim_end().to_string();
    let heading_line = format!("## {heading}");
    if !body.contains(&heading_line) {
        if !body.is_empty() {
            body.push_str("\n\n");
        }
        body.push_str(&heading_line);
    }
    body.push('\n');
    body.push_str(content.trim());
    save_note_body(path, &body).await
}

/// Merge tags into a note frontmatter.
pub async fn update_note_tags(path: &str, tags: &[String]) -> Result<(), String> {
    let content = fs::read_to_string(path).await.map_err(|e| e.to_string())?;
    let (mut meta, body) = parse_frontmatter(&content);
    for tag in tags {
        if !meta.tags.contains(tag) {
            meta.tags.push(tag.to_string());
        }
    }
    meta.tags.sort();
    meta.tags.dedup();
    let frontmatter = build_frontmatter(&meta);
    let content = format!("{frontmatter}\n\n{body}");
    save_atomic(path, &content).await
}

/// Create a new note file with default frontmatter.
pub async fn create_note_file(
    vault_path: &str,
    folder: &str,
    title: &str,
) -> Result<String, String> {
    let safe_title = sanitize_title(title);
    let folder_path = if folder.trim().is_empty() {
        PathBuf::from(vault_path)
    } else {
        Path::new(vault_path).join(folder)
    };
    fs::create_dir_all(&folder_path)
        .await
        .map_err(|e| e.to_string())?;
    let note_path = folder_path.join(format!("{safe_title}.md"));
    let now = Utc::now().to_rfc3339();
    let meta = Frontmatter {
        title: Some(safe_title.clone()),
        created: Some(now.clone()),
        modified: Some(now),
        tags: vec![],
    };
    let content = format!("{}\n\n", build_frontmatter(&meta));
    save_atomic(note_path.to_string_lossy().as_ref(), &content).await?;
    Ok(note_path.to_string_lossy().to_string())
}

/// Delete a file or directory by moving it to trash when possible.
pub async fn delete_path(path: &str) -> Result<(), String> {
    let target = path.to_string();
    let result = tokio::task::spawn_blocking(move || trash::delete(&target))
        .await
        .map_err(|e| e.to_string())?;
    if result.is_ok() {
        return Ok(());
    }
    let metadata = fs::metadata(path).await.map_err(|e| e.to_string())?;
    if metadata.is_dir() {
        fs::remove_dir_all(path).await.map_err(|e| e.to_string())
    } else {
        fs::remove_file(path).await.map_err(|e| e.to_string())
    }
}

/// Rename a file or folder, preserving file extensions when needed.
pub async fn rename_path(old_path: &str, new_name: &str) -> Result<String, String> {
    let old_path = Path::new(old_path);
    let parent = old_path.parent().ok_or("missing parent")?;
    let is_file = fs::metadata(old_path)
        .await
        .map_err(|e| e.to_string())?
        .is_file();
    let mut final_name = new_name.trim().to_string();
    if is_file {
        let ext = old_path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if !ext.is_empty() && !final_name.ends_with(&format!(".{ext}")) {
            final_name = format!("{final_name}.{ext}");
        }
    }
    let new_path = parent.join(final_name);
    fs::rename(old_path, &new_path)
        .await
        .map_err(|e| e.to_string())?;
    Ok(new_path.to_string_lossy().to_string())
}

/// Create a folder under the vault root.
pub async fn create_folder(path: &str) -> Result<(), String> {
    fs::create_dir_all(path).await.map_err(|e| e.to_string())
}

/// Copy an asset into the vault assets directory.
pub async fn import_asset(vault_path: &str, source_path: &str) -> Result<String, String> {
    let source = Path::new(source_path);
    let filename = source
        .file_name()
        .ok_or("invalid source")?
        .to_string_lossy()
        .to_string();
    let ext = source
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    let target_dir = if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp") {
        ".assets/images"
    } else if matches!(ext.as_str(), "wav" | "mp3" | "m4a" | "flac") {
        ".assets/audio"
    } else {
        ".assets/files"
    };
    let target_path = Path::new(vault_path).join(target_dir);
    fs::create_dir_all(&target_path)
        .await
        .map_err(|e| e.to_string())?;
    let target = target_path.join(&filename);
    fs::copy(source_path, &target)
        .await
        .map_err(|e| e.to_string())?;
    Ok(target.to_string_lossy().to_string())
}

/// Resolve an asset by filename across known asset folders.
pub async fn resolve_asset(vault_path: &str, filename: &str) -> Result<String, String> {
    let folders = [
        ".assets/images",
        ".assets/files",
        ".assets/drawings",
        ".assets/audio",
    ];
    for folder in folders {
        let candidate = Path::new(vault_path).join(folder).join(filename);
        if fs::metadata(&candidate).await.is_ok() {
            return Ok(candidate.to_string_lossy().to_string());
        }
    }
    Ok(Path::new(vault_path)
        .join(".assets/files")
        .join(filename)
        .to_string_lossy()
        .to_string())
}

/// Write a file atomically using a temporary file then rename.
pub async fn save_atomic(path: &str, content: &str) -> Result<(), String> {
    let path = Path::new(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, content).await.map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).await.map_err(|e| e.to_string())?;
    Ok(())
}

async fn build_tree_inner(path: &Path) -> Result<Vec<FileNode>, String> {
    let mut nodes = vec![];
    let mut dir = fs::read_dir(path).await.map_err(|e| e.to_string())?;
    while let Some(entry) = dir.next_entry().await.map_err(|e| e.to_string())? {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == ".quillpaw" || name == ".assets" {
            continue;
        }
        let file_type = entry.file_type().await.map_err(|e| e.to_string())?;
        let path = entry.path();
        let is_folder = file_type.is_dir();
        let children = if is_folder {
            Some(build_tree_inner(&path).await?)
        } else {
            None
        };
        nodes.push(FileNode {
            name,
            path: path.to_string_lossy().to_string(),
            is_folder,
            children,
        });
    }
    nodes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(nodes)
}

fn parse_frontmatter(content: &str) -> (Frontmatter, String) {
    let mut meta = Frontmatter::default();
    let mut lines = content.lines();
    let first = lines.next().unwrap_or_default();
    if first.trim() != FRONTMATTER_DELIM {
        return (meta, content.to_string());
    }
    let mut meta_lines = vec![];
    for line in &mut lines {
        if line.trim() == FRONTMATTER_DELIM {
            break;
        }
        meta_lines.push(line);
    }
    for line in meta_lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            let value = strip_quotes(value.trim());
            match key.trim() {
                "title" => meta.title = Some(value),
                "created" => meta.created = Some(value),
                "modified" => meta.modified = Some(value),
                "tags" => meta.tags = parse_list_value(&value),
                _ => {}
            }
        }
    }
    let body = lines.collect::<Vec<_>>().join("\n");
    (meta, body)
}

fn build_frontmatter(meta: &Frontmatter) -> String {
    let title = meta
        .title
        .clone()
        .unwrap_or_else(|| "Untitled".to_string());
    let created = meta
        .created
        .clone()
        .unwrap_or_else(|| Utc::now().to_rfc3339());
    let modified = meta
        .modified
        .clone()
        .unwrap_or_else(|| Utc::now().to_rfc3339());
    let tags = if meta.tags.is_empty() {
        String::new()
    } else {
        meta.tags
            .iter()
            .map(|tag| format_tag(tag))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let tag_line = if tags.is_empty() {
        "tags: []".to_string()
    } else {
        format!("tags: [{tags}]")
    };
    format!(
        "{FRONTMATTER_DELIM}\n\
title: \"{title}\"\n\
created: {created}\n\
modified: {modified}\n\
{tag_line}\n\
{FRONTMATTER_DELIM}"
    )
}

fn strip_quotes(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

fn parse_list_value(value: &str) -> Vec<String> {
    let trimmed = value.trim();
    let content = trimmed
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim();
    if content.is_empty() {
        return vec![];
    }
    content
        .split(',')
        .map(|item| strip_quotes(item.trim()))
        .filter(|item| !item.is_empty())
        .collect()
}

fn format_tag(tag: &str) -> String {
    if tag.contains(' ') || tag.contains(',') {
        format!("\"{tag}\"")
    } else {
        tag.to_string()
    }
}

fn sanitize_title(title: &str) -> String {
    let trimmed = title.trim();
    let cleaned: String = trimmed
        .chars()
        .filter(|c| !matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
        .collect();
    if cleaned.is_empty() {
        "Untitled".to_string()
    } else {
        cleaned
    }
}

fn title_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string()
}
