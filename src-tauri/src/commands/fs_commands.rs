use tauri::AppHandle;


use crate::fs_manager;
use crate::models::{FileNode, NoteContent};
use crate::watcher;

/// Prompt the user to open a vault folder and initialize required structure.
#[tauri::command]
pub async fn open_vault(app: AppHandle) -> Result<String, String> {
    let folder = app.dialog().directory().pick_folder().await.map_err(|e| e.to_string())?;
    let Some(path) = folder else {
        return Err("No vault folder selected".to_string());
    };
    let path_string = path.to_string_lossy().to_string();
    fs_manager::ensure_vault_structure(&path_string).await?;
    watcher::start_watcher(app, path_string.clone()).await?;
    Ok(path_string)
}

/// Return a recursive file tree for the vault path.
#[tauri::command]
pub async fn get_file_tree(vault_path: String) -> Result<Vec<FileNode>, String> {
    fs_manager::build_tree(&vault_path).await
}

/// Read a note file from disk.
#[tauri::command]
pub async fn read_note(path: String) -> Result<NoteContent, String> {
    fs_manager::read_note_file(&path).await
}

/// Save a note body to disk using atomic writes.
#[tauri::command]
pub async fn save_note(path: String, content: String) -> Result<(), String> {
    fs_manager::save_note_body(&path, &content).await
}

/// Create a new note in the given folder.
#[tauri::command]
pub async fn create_note(
    vault_path: String,
    folder: String,
    title: String,
) -> Result<String, String> {
    fs_manager::create_note_file(&vault_path, &folder, &title).await
}

/// Delete a file or folder from the vault.
#[tauri::command]
pub async fn delete_item(path: String) -> Result<(), String> {
    fs_manager::delete_path(&path).await
}

/// Rename a file or folder and return the new path.
#[tauri::command]
pub async fn rename_item(old_path: String, new_name: String) -> Result<String, String> {
    fs_manager::rename_path(&old_path, &new_name).await
}

/// Create a folder within the vault.
#[tauri::command]
pub async fn create_folder(vault_path: String, folder_name: String) -> Result<(), String> {
    let path = std::path::Path::new(&vault_path).join(folder_name);
    fs_manager::create_folder(path.to_string_lossy().as_ref()).await
}

/// Import a file into the vault assets folder.
#[tauri::command]
pub async fn import_asset(vault_path: String, source_path: String) -> Result<String, String> {
    fs_manager::import_asset(&vault_path, &source_path).await
}

/// Resolve an asset filename to its full path within the vault.
#[tauri::command]
pub async fn resolve_asset(vault_path: String, filename: String) -> Result<String, String> {
    fs_manager::resolve_asset(&vault_path, &filename).await
}
