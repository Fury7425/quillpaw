use tokio::fs;

pub async fn save(vault_path: &str, filename: &str, drawing_json: &str) -> Result<String, String> {
    let p = format!("{vault_path}/.assets/drawings/{filename}");
    fs::create_dir_all(format!("{vault_path}/.assets/drawings")).await.map_err(|e| e.to_string())?;
    fs::write(&p, drawing_json).await.map_err(|e| e.to_string())?;
    Ok(p)
}

pub async fn load(vault_path: &str, filename: &str) -> Result<String, String> {
    let p = format!("{vault_path}/.assets/drawings/{filename}");
    fs::read_to_string(p).await.map_err(|e| e.to_string())
}
