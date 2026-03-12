use crate::{models::SearchResult, search};

#[tauri::command]
pub async fn build_search_index(vault_path: String) -> Result<(), String> { search::build_index(&vault_path).await }
#[tauri::command]
pub async fn search_notes(vault_path: String, query: String) -> Result<Vec<SearchResult>, String> { search::keyword_search(&vault_path, &query).await }
#[tauri::command]
pub async fn search_semantic(vault_path: String, query: String) -> Result<Vec<SearchResult>, String> { search::semantic_search(&vault_path, &query).await }
