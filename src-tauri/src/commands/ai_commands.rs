use serde_json::json;
use tauri::AppHandle;

use crate::{
    ai_engine::{self, AiDeviceMode},
    embeddings, fs_manager,
    models::{AiModelStatus, AiProposal},
    search,
};

/// Load an AI model from disk.
#[tauri::command]
pub async fn load_ai_model(
    model_path: String,
    device_mode: String,
) -> Result<AiModelStatus, String> {
    let mode = parse_device_mode(&device_mode)?;
    ai_engine::load_model(&model_path, mode).await
}

/// Unload the currently loaded AI model.
#[tauri::command]
pub async fn unload_ai_model() -> Result<AiModelStatus, String> {
    ai_engine::unload_model().await
}

/// Get the current AI model status.
#[tauri::command]
pub async fn get_ai_status() -> Result<AiModelStatus, String> {
    ai_engine::model_status().await
}

/// Detect if an NPU is available.
#[tauri::command]
pub async fn detect_npu() -> Result<bool, String> {
    ai_engine::npu_available().await
}

/// Download a model into the vault model directory.
#[tauri::command]
pub async fn download_ai_model(
    app: AppHandle,
    vault_path: String,
    model_id: String,
    url: Option<String>,
) -> Result<String, String> {
    ai_engine::download_model(app, &vault_path, &model_id, url).await
}

/// Summarize a note body and return a proposal.
#[tauri::command]
pub async fn summarize_note(
    note_content: String,
    target_path: String,
) -> Result<AiProposal, String> {
    ensure_model_loaded().await?;
    let summary = ai_engine::prompt(
        "You are a helpful assistant that summarizes notes. Provide a concise 2-3 sentence summary.",
        &format!("Summarize this note:\n\n{note_content}")
    ).await?;
    ai_engine::make_proposal("summary", "Summary", &summary, Some(target_path), None).await
}

/// Answer a question based on the vault context.
#[tauri::command]
pub async fn ask_question(
    vault_path: String,
    question: String,
    target_path: Option<String>,
) -> Result<AiProposal, String> {
    ensure_model_loaded().await?;
    let results = search::keyword_search(&vault_path, &question).await?;
    let mut context_text = String::new();
    for result in results.iter().take(3) {
        let content = fs_manager::read_note_file(&result.path).await?;
        context_text.push_str(&format!(
            "\nNote: {}\nContent: {}\n",
            result.title, content.body
        ));
    }

    let response = ai_engine::prompt(
        "You are a helpful assistant that answers questions based ONLY on the provided notes. If the answer is not in the notes, say you don't know.",
        &format!("Context: {context_text}\n\nQuestion: {question}")
    ).await?;

    let metadata = json!({
        "sources": results.iter().take(5).map(|r| json!({
            "path": r.path,
            "title": r.title,
            "score": r.score
        })).collect::<Vec<_>>()
    });
    ai_engine::make_proposal("qa", "Answer", &response, target_path, Some(metadata)).await
}

/// Detect reminders in a note body.
#[tauri::command]
pub async fn detect_reminders(
    note_content: String,
    target_path: String,
) -> Result<Vec<AiProposal>, String> {
    ensure_model_loaded().await?;
    let reminders = extract_reminders(&note_content);
    let mut proposals = Vec::new();
    for reminder in reminders {
        proposals.push(
            ai_engine::make_proposal(
                "reminder",
                "Reminder",
                &reminder,
                Some(target_path.clone()),
                Some(json!({ "reminder": reminder })),
            )
            .await?,
        );
    }
    Ok(proposals)
}

/// Suggest tags for a note.
#[tauri::command]
pub async fn suggest_tags(
    vault_path: String,
    note_content: String,
    target_path: String,
) -> Result<AiProposal, String> {
    ensure_model_loaded().await?;
    let mut tags = embeddings::suggest_tags(&vault_path, &target_path, &note_content)
        .await
        .unwrap_or_default();
    if tags.is_empty() {
        tags = extract_keywords(&note_content);
    }
    let content = if tags.is_empty() {
        "No tag suggestions found.".to_string()
    } else {
        format!("Suggested tags: {}", tags.join(", "))
    };
    let metadata = json!({ "tags": tags });
    ai_engine::make_proposal(
        "tags",
        "Tag Suggestions",
        &content,
        Some(target_path),
        Some(metadata),
    )
    .await
}

/// Apply an AI proposal to a target note.
#[tauri::command]
pub async fn apply_ai_proposal(proposal_id: String, target_path: String) -> Result<(), String> {
    let proposal = ai_engine::take_proposal(&proposal_id).await?;
    match proposal.proposal_type.as_str() {
        "tags" => {
            let tags = proposal
                .metadata
                .as_ref()
                .and_then(|value| value.get("tags"))
                .and_then(|value| value.as_array())
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if !tags.is_empty() {
                fs_manager::update_note_tags(&target_path, &tags).await?;
            }
        }
        "reminder" => {
            let reminder = proposal.content.trim();
            let body = format!("- [ ] {reminder}");
            fs_manager::append_note_section(&target_path, "Reminders", &body).await?;
        }
        "summary" => {
            fs_manager::append_note_section(&target_path, "Summary", &proposal.content).await?;
        }
        "qa" => {
            fs_manager::append_note_section(&target_path, "Answer", &proposal.content).await?;
        }
        _ => {
            fs_manager::append_note_section(&target_path, "AI Notes", &proposal.content).await?;
        }
    }
    Ok(())
}

fn parse_device_mode(input: &str) -> Result<AiDeviceMode, String> {
    match input.trim().to_lowercase().as_str() {
        "auto" => Ok(AiDeviceMode::Auto),
        "cpu" => Ok(AiDeviceMode::Cpu),
        "npu" | "low (npu)" | "low-npu" => Ok(AiDeviceMode::Npu),
        _ => Err("Unknown device mode.".to_string()),
    }
}

async fn ensure_model_loaded() -> Result<(), String> {
    if ai_engine::is_model_loaded().await? {
        Ok(())
    } else {
        Err("AI model not loaded. Enable AI in Settings.".to_string())
    }
}

fn build_summary(note_content: &str) -> String {
    let sentences = note_content
        .split(|c| c == '.' || c == '!' || c == '?')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .take(4)
        .collect::<Vec<_>>();
    if sentences.is_empty() {
        "No content available to summarize.".to_string()
    } else {
        sentences.join(". ") + "."
    }
}

fn extract_reminders(note_content: &str) -> Vec<String> {
    let mut reminders = vec![];
    for line in note_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- [ ]")
            || trimmed.to_lowercase().contains("remind")
            || trimmed.to_lowercase().contains("todo")
        {
            reminders.push(trimmed.trim_start_matches("- [ ]").trim().to_string());
        }
    }
    reminders
}

fn extract_keywords(note_content: &str) -> Vec<String> {
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for word in note_content
        .split(|c: char| !c.is_alphanumeric() && c != '-')
        .map(|w| w.to_lowercase())
    {
        if word.len() < 4 {
            continue;
        }
        *counts.entry(word).or_insert(0) += 1;
    }
    let mut sorted = counts.into_iter().collect::<Vec<_>>();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.into_iter().take(6).map(|(word, _)| word).collect()
}
