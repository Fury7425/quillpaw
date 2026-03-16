use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

use chrono::Utc;
use futures_util::StreamExt;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::model::params::LlamaModelParams;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::models::{AiModelStatus, AiProposal};

static PROPOSALS: OnceLock<Mutex<HashMap<String, AiProposal>>> = OnceLock::new();
static AI_STATE: OnceLock<Mutex<AiState>> = OnceLock::new();

#[derive(Clone, Copy, Debug)]
pub enum AiDeviceMode {
    Auto,
    Cpu,
    Npu,
}

impl AiDeviceMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            AiDeviceMode::Auto => "auto",
            AiDeviceMode::Cpu => "cpu",
            AiDeviceMode::Npu => "npu",
        }
    }
}

use llama_cpp_2::token::data_array::LlamaTokenDataArray;
use llama_cpp_2::token::LlamaToken;

struct LoadedModel {
    model_path: String,
    backend: LlamaBackend,
    model: LlamaModel,
    device_mode: AiDeviceMode,
    #[allow(dead_code)]
    loaded_at: String,
}

#[derive(Default)]
struct AiState {
    model: Option<LoadedModel>,
    npu_available: Option<bool>,
}

fn state() -> &'static Mutex<AiState> {
    AI_STATE.get_or_init(|| Mutex::new(AiState::default()))
}

/// Run inference with the active model.
pub async fn prompt(system: &str, user: &str) -> Result<String, String> {
    let system_owned = system.to_string();
    let user_owned = user.to_string();

    // We need to do all llama work inside spawn_blocking since the types are !Send.
    // Re-acquire the lock inside the blocking context.
    tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Handle::current();
        let guard = rt.block_on(state().lock());
        let loaded = guard.model.as_ref().ok_or("Model not loaded")?;

        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(Some(std::num::NonZeroU32::new(2048).unwrap()));
        let mut context = loaded.model.new_context(&loaded.backend, ctx_params)
            .map_err(|e| e.to_string())?;

        let prompt_text = format!("<|system|>\n{}\n<|user|>\n{}\n<|assistant|>\n", system_owned, user_owned);
        let tokens = loaded.model.str_to_token(&prompt_text, llama_cpp_2::model::AddBos::Always).map_err(|e| e.to_string())?;

        let mut batch = llama_cpp_2::llama_batch::LlamaBatch::new(tokens.len() + 512, 1);
        for (i, &token) in tokens.iter().enumerate() {
            let last = i == tokens.len() - 1;
            batch.add(token, i as i32, &[0i32], last).map_err(|e| e.to_string())?;
        }

        context.decode(&mut batch).map_err(|e| e.to_string())?;

        let mut output = String::new();
        let mut n_cur = tokens.len();
        while n_cur < 512 {
            let logits = context.get_logits_ith(batch.n_tokens() - 1);
            let mut candidates = LlamaTokenDataArray::from_iter(
                logits.iter().enumerate().map(|(i, &logit)| {
                    llama_cpp_2::token::data::LlamaTokenData::new(LlamaToken(i as i32), logit, 0.0)
                }),
                false,
            );

            let token = candidates.sample_token_greedy();
            if token == loaded.model.token_eos() {
                break;
            }

            let piece_bytes = loaded.model.token_to_bytes(token, llama_cpp_2::model::Special::Tokenize).map_err(|e| e.to_string())?;
            if let Ok(piece_str) = String::from_utf8(piece_bytes) {
                output.push_str(&piece_str);
            }

            batch.clear();
            batch.add(token, n_cur as i32, &[0i32], true).map_err(|e| e.to_string())?;
            context.decode(&mut batch).map_err(|e| e.to_string())?;
            n_cur += 1;
        }

        Ok(output)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Load a GGUF model into the AI runtime.
pub async fn load_model(model_path: &str, device_mode: AiDeviceMode) -> Result<AiModelStatus, String> {
    let path = Path::new(model_path);
    if !path.exists() {
        return Err("Model file not found.".to_string());
    }
    if matches!(device_mode, AiDeviceMode::Npu) && !npu_available().await? {
        return Err("NPU mode selected but no NPU detected.".to_string());
    }
    let model_path = model_path.to_string();
    let loaded = tokio::task::spawn_blocking(move || -> Result<LoadedModel, String> {
        let backend = LlamaBackend::init().map_err(|e| e.to_string())?;
        let model_params = LlamaModelParams::default();
        if matches!(device_mode, AiDeviceMode::Npu) {
            // Placeholder for NPU specific params if needed
        }
        let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)
            .map_err(|e| e.to_string())?;
        Ok(LoadedModel {
            model_path,
            backend,
            model,
            device_mode,
            loaded_at: Utc::now().to_rfc3339(),
        })
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut guard = state().lock().await;
    guard.model = Some(loaded);
    Ok(build_status(&guard, "Model loaded."))
}

/// Unload the active AI model.
pub async fn unload_model() -> Result<AiModelStatus, String> {
    let mut guard = state().lock().await;
    guard.model = None;
    Ok(build_status(&guard, "Model unloaded."))
}

/// Retrieve current AI model status.
pub async fn model_status() -> Result<AiModelStatus, String> {
    let guard = state().lock().await;
    Ok(build_status(&guard, "Status ready."))
}

/// Check if a model is currently loaded.
pub async fn is_model_loaded() -> Result<bool, String> {
    let guard = state().lock().await;
    Ok(guard.model.is_some())
}

/// Detect whether an Intel NPU is available.
pub async fn npu_available() -> Result<bool, String> {
    let mut guard = state().lock().await;
    if let Some(value) = guard.npu_available {
        return Ok(value);
    }
    let detected = tokio::task::spawn_blocking(detect_npu_blocking)
        .await
        .map_err(|e| e.to_string())??;
    guard.npu_available = Some(detected);
    Ok(detected)
}

/// Build a lightweight AI proposal payload.
pub async fn make_proposal(
    proposal_type: &str,
    title: &str,
    content: &str,
    target_path: Option<String>,
    metadata: Option<Value>,
) -> Result<AiProposal, String> {
    let proposal = AiProposal {
        id: Uuid::new_v4().to_string(),
        proposal_type: proposal_type.to_string(),
        title: title.to_string(),
        content: content.to_string(),
        target_path,
        metadata,
    };
    let store = PROPOSALS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = store.lock().await;
    guard.insert(proposal.id.clone(), proposal.clone());
    Ok(proposal)
}

/// Take a stored proposal by id.
pub async fn take_proposal(proposal_id: &str) -> Result<AiProposal, String> {
    let store = PROPOSALS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = store.lock().await;
    guard
        .remove(proposal_id)
        .ok_or_else(|| "proposal not found".to_string())
}

/// Download a model into the vault model directory with progress events.
pub async fn download_model(
    app: AppHandle,
    vault_path: &str,
    model_id: &str,
    custom_url: Option<String>,
) -> Result<String, String> {
    let spec = resolve_model_spec(model_id, custom_url)?;
    let target_dir = Path::new(vault_path).join(".quillpaw/models");
    tokio::fs::create_dir_all(&target_dir)
        .await
        .map_err(|e| e.to_string())?;
    let target_path = target_dir.join(&spec.filename);
    let client = reqwest::Client::new();
    let response = client
        .get(&spec.url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    let total = response.content_length();
    let mut file = tokio::fs::File::create(&target_path)
        .await
        .map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        emit_download_progress(
            &app,
            ModelDownloadProgress {
                model_id: model_id.to_string(),
                downloaded,
                total,
                done: false,
                path: None,
                error: None,
            },
        )?;
    }
    emit_download_progress(
        &app,
        ModelDownloadProgress {
            model_id: model_id.to_string(),
            downloaded,
            total,
            done: true,
            path: Some(target_path.to_string_lossy().to_string()),
            error: None,
        },
    )?;
    Ok(target_path.to_string_lossy().to_string())
}

fn build_status(state: &AiState, detail: &str) -> AiModelStatus {
    let (loaded, model_path, device_mode) = match &state.model {
        Some(model) => (true, Some(model.model_path.clone()), model.device_mode),
        None => (false, None, AiDeviceMode::Auto),
    };
    AiModelStatus {
        loaded,
        model_path,
        device_mode: device_mode.as_str().to_string(),
        npu_available: state.npu_available.unwrap_or(false),
        detail: detail.to_string(),
    }
}

#[derive(Clone)]
struct ModelSpec {
    id: String,
    filename: String,
    url: String,
}

#[derive(Serialize, Deserialize)]
struct ModelDownloadProgress {
    model_id: String,
    downloaded: u64,
    total: Option<u64>,
    done: bool,
    path: Option<String>,
    error: Option<String>,
}

fn resolve_model_spec(model_id: &str, custom_url: Option<String>) -> Result<ModelSpec, String> {
    let specs = [
        (
            "phi-3-mini",
            "phi-3-mini.gguf",
            "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4.gguf",
        ),
        (
            "qwen2.5-1.5b",
            "qwen2.5-1.5b.gguf",
            "https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/resolve/main/Qwen2.5-1.5B-Instruct-Q4_K_M.gguf",
        ),
    ];
    if let Some(url) = custom_url {
        let filename = Path::new(&url)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(model_id)
            .to_string();
        return Ok(ModelSpec {
            id: model_id.to_string(),
            filename,
            url,
        });
    }
    for (id, filename, url) in specs {
        if id == model_id {
            return Ok(ModelSpec {
                id: id.to_string(),
                filename: filename.to_string(),
                url: url.to_string(),
            });
        }
    }
    Err("Unknown model id.".to_string())
}

fn emit_download_progress(app: &AppHandle, progress: ModelDownloadProgress) -> Result<(), String> {
    app.emit("model-download-progress", progress)
        .map_err(|e| e.to_string())
}

#[cfg(target_os = "windows")]
fn detect_npu_blocking() -> Result<bool, String> {
    use serde::Deserialize;
    use wmi::{COMLibrary, WMIConnection};

    #[derive(Deserialize)]
    struct PnpEntity {
        #[serde(rename = "Name")]
        name: Option<String>,
    }

    let com_lib = COMLibrary::new().map_err(|e| e.to_string())?;
    let wmi_con = WMIConnection::new(com_lib.into()).map_err(|e| e.to_string())?;
    let query = "SELECT Name FROM Win32_PnPEntity WHERE Name LIKE '%NPU%' OR Name LIKE '%AI Boost%'";
    let results: Vec<PnpEntity> = wmi_con.raw_query(query).map_err(|e| e.to_string())?;
    Ok(results.iter().any(|item| {
        item.name
            .as_deref()
            .unwrap_or_default()
            .to_lowercase()
            .contains("npu")
            || item
                .name
                .as_deref()
                .unwrap_or_default()
                .to_lowercase()
                .contains("ai boost")
    }))
}

#[cfg(not(target_os = "windows"))]
fn detect_npu_blocking() -> Result<bool, String> {
    Ok(false)
}
