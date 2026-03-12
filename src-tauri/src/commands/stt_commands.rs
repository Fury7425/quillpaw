use tauri::AppHandle;

use crate::stt_engine;

#[tauri::command]
pub async fn start_lecture_mode(_app: AppHandle) -> Result<(), String> { stt_engine::start().await }
#[tauri::command]
pub async fn stop_lecture_mode() -> Result<(), String> { stt_engine::stop().await }
#[tauri::command]
pub async fn list_audio_devices() -> Result<Vec<String>, String> { Ok(vec!["Default".into()]) }
#[tauri::command]
pub async fn set_audio_device(_device_name: String) -> Result<(), String> { Ok(()) }
