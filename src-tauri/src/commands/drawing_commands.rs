use crate::drawing;

#[tauri::command]
pub async fn render_drawing_png(_drawing_json: String) -> Result<Vec<u8>, String> { Ok(vec![]) }
#[tauri::command]
pub async fn save_drawing(vault_path: String, filename: String, drawing_json: String) -> Result<String, String> { drawing::save(&vault_path, &filename, &drawing_json).await }
#[tauri::command]
pub async fn load_drawing(vault_path: String, filename: String) -> Result<String, String> { drawing::load(&vault_path, &filename).await }
