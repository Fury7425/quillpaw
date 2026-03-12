use chrono::Utc;
use tokio::fs;

use crate::models::{FileNode, NoteContent};

pub async fn build_tree(path: &str) -> Result<Vec<FileNode>, String> {
    let mut nodes = vec![];
    let mut dir = fs::read_dir(path).await.map_err(|e| e.to_string())?;
    while let Some(entry) = dir.next_entry().await.map_err(|e| e.to_string())? {
        let p = entry.path();
        let is_folder = p.is_dir();
        nodes.push(FileNode {
            name: entry.file_name().to_string_lossy().to_string(),
            path: p.to_string_lossy().to_string(),
            is_folder,
            children: Some(vec![]),
        });
    }
    Ok(nodes)
}

pub async fn read_note_file(path: &str) -> Result<NoteContent, String> {
    let body = fs::read_to_string(path).await.map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    let title = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string();
    Ok(NoteContent {
        path: path.to_string(),
        title,
        body,
        tags: vec![],
        created: now.clone(),
        modified: now,
    })
}

pub async fn save_atomic(path: &str, content: &str) -> Result<(), String> {
    let tmp = format!("{path}.tmp");
    fs::write(&tmp, content).await.map_err(|e| e.to_string())?;
    fs::rename(tmp, path).await.map_err(|e| e.to_string())?;
    Ok(())
}
