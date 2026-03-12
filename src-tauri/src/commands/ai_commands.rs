use crate::{ai_engine, fs_manager, models::AiProposal};

#[tauri::command]
pub async fn load_ai_model(_model_path: String) -> Result<(), String> { Ok(()) }
#[tauri::command]
pub async fn unload_ai_model() -> Result<(), String> { Ok(()) }
#[tauri::command]
pub async fn summarize_note(note_content: String) -> Result<AiProposal, String> { ai_engine::make_proposal("summary", "Summary", &note_content.chars().take(300).collect::<String>()).await }
#[tauri::command]
pub async fn ask_question(_vault_path: String, question: String) -> Result<AiProposal, String> { ai_engine::make_proposal("qa", "Answer", &format!("Question received: {question}")).await }
#[tauri::command]
pub async fn detect_reminders(note_content: String) -> Result<Vec<AiProposal>, String> { Ok(vec![ai_engine::make_proposal("reminder", "Reminder", &note_content).await?]) }
#[tauri::command]
pub async fn suggest_tags(note_content: String) -> Result<AiProposal, String> { ai_engine::make_proposal("tags", "Tag Suggestions", &note_content).await }
#[tauri::command]
pub async fn apply_ai_proposal(proposal_id: String, target_path: String) -> Result<(), String> {
    fs_manager::save_atomic(&target_path, &format!("<!-- Applied proposal: {proposal_id} -->\n")).await
}
