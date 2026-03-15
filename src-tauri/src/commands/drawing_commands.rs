use crate::drawing;

/// Render a drawing JSON payload into a PNG buffer.
#[tauri::command]
pub async fn render_drawing_png(drawing_json: String) -> Result<Vec<u8>, String> { drawing::render_png(&drawing_json).await }
/// Save a drawing JSON payload to disk.
#[tauri::command]
pub async fn save_drawing(vault_path: String, filename: String, drawing_json: String) -> Result<String, String> { drawing::save(&vault_path, &filename, &drawing_json).await }
/// Load a drawing JSON payload from disk.
#[tauri::command]
pub async fn load_drawing(vault_path: String, filename: String) -> Result<String, String> { drawing::load(&vault_path, &filename).await }
