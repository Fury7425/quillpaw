use crate::{embeddings, models::SearchResult, search};

/// Build the keyword search index for a vault.
#[tauri::command]
pub async fn build_search_index(vault_path: String) -> Result<(), String> {
    search::build_index(&vault_path).await
}
/// Build semantic embeddings for a vault.
#[tauri::command]
pub async fn build_embeddings(vault_path: String) -> Result<(), String> {
    embeddings::build_embeddings(&vault_path).await
}
/// Run a keyword search query.
#[tauri::command]
pub async fn search_notes(vault_path: String, query: String) -> Result<Vec<SearchResult>, String> {
    search::keyword_search(&vault_path, &query).await
}
/// Run a semantic search query.
#[tauri::command]
pub async fn search_semantic(
    vault_path: String,
    query: String,
) -> Result<Vec<SearchResult>, String> {
    search::semantic_search(&vault_path, &query).await
}
