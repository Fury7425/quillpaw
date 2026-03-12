use tauri::AppHandle;
use tokio::fs;

use crate::{fs_manager, models::{FileNode, NoteContent}};

#[tauri::command]
pub async fn open_vault(_app: AppHandle) -> Result<String, String> { Ok(String::new()) }
#[tauri::command]
pub async fn get_file_tree(vault_path: String) -> Result<Vec<FileNode>, String> { fs_manager::build_tree(&vault_path).await }
#[tauri::command]
pub async fn read_note(path: String) -> Result<NoteContent, String> { fs_manager::read_note_file(&path).await }
#[tauri::command]
pub async fn save_note(path: String, content: String) -> Result<(), String> { fs_manager::save_atomic(&path, &content).await }
#[tauri::command]
pub async fn create_note(vault_path: String, folder: String, title: String) -> Result<String, String> {
    let path = format!("{vault_path}/{folder}/{title}.md");
    fs::write(&path, "").await.map_err(|e| e.to_string())?;
    Ok(path)
}
#[tauri::command]
pub async fn delete_item(path: String) -> Result<(), String> { fs::remove_file(path).await.map_err(|e| e.to_string()) }
#[tauri::command]
pub async fn rename_item(old_path: String, new_name: String) -> Result<String, String> {
    let parent = std::path::Path::new(&old_path).parent().ok_or("missing parent")?;
    let new_path = parent.join(new_name);
    fs::rename(&old_path, &new_path).await.map_err(|e| e.to_string())?;
    Ok(new_path.to_string_lossy().to_string())
}
#[tauri::command]
pub async fn create_folder(vault_path: String, folder_name: String) -> Result<(), String> {
    fs::create_dir_all(format!("{vault_path}/{folder_name}")).await.map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn import_asset(vault_path: String, source_path: String) -> Result<String, String> {
    let file = std::path::Path::new(&source_path).file_name().ok_or("invalid source")?.to_string_lossy();
    let target_dir = format!("{vault_path}/.assets/files");
    fs::create_dir_all(&target_dir).await.map_err(|e| e.to_string())?;
    let target = format!("{target_dir}/{file}");
    fs::copy(source_path, &target).await.map_err(|e| e.to_string())?;
    Ok(target)
}
#[tauri::command]
pub async fn resolve_asset(vault_path: String, filename: String) -> Result<String, String> {
    Ok(format!("{vault_path}/.assets/files/{filename}"))
}
