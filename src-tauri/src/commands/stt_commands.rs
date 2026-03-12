use tauri::AppHandle;

use crate::stt_engine;

/// Start lecture mode capture.
#[tauri::command]
pub async fn start_lecture_mode(app: AppHandle, vault_path: String) -> Result<(), String> {
    stt_engine::start(app, vault_path).await
}

/// Stop lecture mode capture.
#[tauri::command]
pub async fn stop_lecture_mode() -> Result<(), String> {
    stt_engine::stop().await
}

/// List available audio devices.
#[tauri::command]
pub async fn list_audio_devices() -> Result<Vec<String>, String> {
    stt_engine::list_devices().await
}

/// Set the active audio input device.
#[tauri::command]
pub async fn set_audio_device(device_name: String) -> Result<(), String> {
    stt_engine::set_device(device_name).await
}

/// Set the speech model path.
#[tauri::command]
pub async fn set_stt_model_path(model_path: String) -> Result<(), String> {
    stt_engine::set_model_path(model_path).await
}
