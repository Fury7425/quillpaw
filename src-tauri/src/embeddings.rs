use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::UNIX_EPOCH;

use ndarray::{Array2, Axis};
use ort::{
    inputs,
    session::{builder::GraphOptimizationLevel, Session},
};
use serde::{Deserialize, Serialize};
use tokenizers::Tokenizer;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use walkdir::WalkDir;

use crate::models::SearchResult;

const EMBEDDING_DIM: usize = 384;
const MANIFEST_FILE: &str = "manifest.json";
const HNSW_FILE: &str = "hnsw.bin";

const MODEL_URL: &str = "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx";
const TOKENIZER_URL: &str = "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json";

static MODEL_MANAGER: OnceLock<Mutex<ModelManager>> = OnceLock::new();

struct ModelManager {
    session: Option<Session>,
    tokenizer: Option<Tokenizer>,
}

impl ModelManager {
    async fn get() -> &'static Mutex<Self> {
        MODEL_MANAGER.get_or_init(|| Mutex::new(Self { session: None, tokenizer: None }))
    }

    async fn ensure_loaded(&mut self, vault_path: &str) -> Result<(), String> {
        if self.session.is_some() && self.tokenizer.is_some() {
            return Ok(());
        }

        let model_dir = Path::new(vault_path).join(".quillpaw/models/embeddings");
        tokio::fs::create_dir_all(&model_dir).await.map_err(|e| e.to_string())?;

        let model_path = model_dir.join("model.onnx");
        let tokenizer_path = model_dir.join("tokenizer.json");

        if !model_path.exists() {
            download_file(MODEL_URL, &model_path).await?;
        }
        if !tokenizer_path.exists() {
            download_file(TOKENIZER_URL, &tokenizer_path).await?;
        }

        let tokenizer = Tokenizer::from_file(&tokenizer_path).map_err(|e| e.to_string())?;
        let session = Session::builder()
            .map_err(|e| e.to_string())?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| e.to_string())?
            .with_intra_threads(4)
            .map_err(|e| e.to_string())?
            .commit_from_file(&model_path)
            .map_err(|e| e.to_string())?;

        self.tokenizer = Some(tokenizer);
        self.session = Some(session);
        Ok(())
    }

    async fn embed(&mut self, text: &str) -> Result<Vec<f32>, String> {
        let tokenizer = self.tokenizer.as_ref().ok_or("Tokenizer not loaded")?;
        let session = self.session.as_ref().ok_or("Session not loaded")?;

        let encoding = tokenizer.encode(text, true).map_err(|e| e.to_string())?;
        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&id| id as i64).collect();
        let attention_mask: Vec<i64> = encoding.get_attention_mask().iter().map(|&mask| mask as i64).collect();
        let token_type_ids: Vec<i64> = encoding.get_type_ids().iter().map(|&id| id as i64).collect();

        let batch_size = 1;
        let seq_len = input_ids.len();

        let input_ids_array = Array2::from_shape_vec((batch_size, seq_len), input_ids).map_err(|e| e.to_string())?;
        let attention_mask_array = Array2::from_shape_vec((batch_size, seq_len), attention_mask).map_err(|e| e.to_string())?;
        let token_type_ids_array = Array2::from_shape_vec((batch_size, seq_len), token_type_ids).map_err(|e| e.to_string())?;

        let outputs = session.run(inputs![
            "input_ids" => input_ids_array.view(),
            "attention_mask" => attention_mask_array.view(),
            "token_type_ids" => token_type_ids_array.view(),
        ].unwrap()).map_err(|e| e.to_string())?;

        let (_shape, data) = outputs["last_hidden_state"].try_extract_tensor::<f32>().map_err(|e| e.to_string())?;
        
        // Mean pooling
        let mut mean_embedding = vec![0.0; EMBEDDING_DIM];
        for i in 0..seq_len {
            for j in 0..EMBEDDING_DIM {
                mean_embedding[j] += data[i * EMBEDDING_DIM + j];
            }
        }
        for j in 0..EMBEDDING_DIM {
            mean_embedding[j] /= seq_len as f32;
        }

        normalize(&mut mean_embedding);
        Ok(mean_embedding)
    }
}

async fn download_file(url: &str, path: &Path) -> Result<(), String> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    let mut file = tokio::fs::File::create(path).await.map_err(|e| e.to_string())?;
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    file.write_all(&bytes).await.map_err(|e| e.to_string())
}

#[derive(Serialize, Deserialize, Clone)]
struct EmbeddingEntry {
    id: u64,
    path: String,
    modified: u64,
    embedding_file: String,
    title: String,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
struct EmbeddingManifest {
    dim: usize,
    entries: Vec<EmbeddingEntry>,
}

pub async fn build_embeddings(vault_path: &str) -> Result<(), String> {
    let mut manager = ModelManager::get().await.lock().await;
    manager.ensure_loaded(vault_path).await?;

    let base_dir = embeddings_dir(vault_path);
    tokio::fs::create_dir_all(&base_dir).await.map_err(|e| e.to_string())?;
    let manifest_path = base_dir.join(MANIFEST_FILE);
    let mut manifest = load_manifest(&manifest_path).await?;
    let mut existing: HashMap<String, EmbeddingEntry> = manifest
        .entries
        .drain(..)
        .map(|entry| (entry.path.clone(), entry))
        .collect();

    for entry in WalkDir::new(vault_path).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() { continue; }
        let path = entry.path();
        if should_skip(path) || path.extension().and_then(|s| s.to_str()) != Some("md") { continue; }
        let modified = file_modified(path).await?;
        let path_str = path.to_string_lossy().to_string();
        let mut cached = existing.remove(&path_str);
        if let Some(existing_entry) = &cached {
            if existing_entry.modified == modified {
                manifest.entries.push(existing_entry.clone());
                continue;
            }
        }
        let content = tokio::fs::read_to_string(path).await.map_err(|e| e.to_string())?;
        let (title, tags, body) = parse_frontmatter(&content, path);
        
        let embedding = manager.embed(&body).await?;
        
        let embedding_file = cached.take().map(|entry| entry.embedding_file).unwrap_or_else(|| embedding_filename(&path_str));
        let embedding_path = base_dir.join(&embedding_file);
        save_embedding(&embedding_path, &embedding).await?;
        let entry = EmbeddingEntry {
            id: hash_id(&path_str),
            path: path_str.clone(),
            modified,
            embedding_file,
            title,
            tags,
        };
        manifest.entries.push(entry);
    }

    manifest.dim = EMBEDDING_DIM;
    manifest.entries.sort_by(|a, b| a.path.cmp(&b.path));
    let content = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
    let mut file = tokio::fs::File::create(&manifest_path).await.map_err(|e| e.to_string())?;
    file.write_all(&content).await.map_err(|e| e.to_string())?;
    build_hnsw_index(&base_dir, &manifest).await?;
    Ok(())
}

pub async fn semantic_search(vault_path: &str, query: &str) -> Result<Vec<SearchResult>, String> {
    let mut manager = ModelManager::get().await.lock().await;
    manager.ensure_loaded(vault_path).await?;

    let base_dir = embeddings_dir(vault_path);
    let manifest = load_manifest(&base_dir.join(MANIFEST_FILE)).await?;
    if manifest.entries.is_empty() { return Ok(vec![]); }
    
    let query_embedding = manager.embed(query).await?;
    
    let mut results = match search_hnsw(&base_dir, &manifest, &query_embedding).await {
        Ok(matches) => matches,
        Err(_) => brute_force_search(&manifest, &base_dir, &query_embedding).await?,
    };
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(results)
}

pub async fn suggest_tags(vault_path: &str, note_path: &str, note_content: &str) -> Result<Vec<String>, String> {
    let mut manager = ModelManager::get().await.lock().await;
    manager.ensure_loaded(vault_path).await?;

    let base_dir = embeddings_dir(vault_path);
    let manifest = load_manifest(&base_dir.join(MANIFEST_FILE)).await?;
    if manifest.entries.is_empty() { return Ok(vec![]); }
    
    let query_embedding = manager.embed(note_content).await?;
    
    let mut matches = search_hnsw(&base_dir, &manifest, &query_embedding).await?;
    matches.retain(|item| item.path != note_path);
    let mut counts: HashMap<String, usize> = HashMap::new();
    for entry in matches.iter().take(8) {
        if let Some(found) = manifest.entries.iter().find(|e| e.path == entry.path) {
            for tag in &found.tags {
                *counts.entry(tag.to_string()).or_insert(0) += 1;
            }
        }
    }
    let mut sorted = counts.into_iter().collect::<Vec<_>>();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(sorted.into_iter().take(6).map(|(tag, _)| tag).collect())
}

async fn load_manifest(path: &Path) -> Result<EmbeddingManifest, String> {
    if tokio::fs::metadata(path).await.is_err() { return Ok(EmbeddingManifest::default()); }
    let content = tokio::fs::read_to_string(path).await.map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

async fn save_embedding(path: &Path, embedding: &[f32]) -> Result<(), String> {
    let content = serde_json::to_vec(embedding).map_err(|e| e.to_string())?;
    let mut file = tokio::fs::File::create(path).await.map_err(|e| e.to_string())?;
    file.write_all(&content).await.map_err(|e| e.to_string())
}

async fn read_embedding(path: &Path) -> Result<Vec<f32>, String> {
    let content = tokio::fs::read_to_string(path).await.map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

async fn brute_force_search(manifest: &EmbeddingManifest, base_dir: &Path, query_embedding: &[f32]) -> Result<Vec<SearchResult>, String> {
    let mut results = vec![];
    for entry in &manifest.entries {
        let embedding_path = base_dir.join(&entry.embedding_file);
        let embedding = read_embedding(&embedding_path).await?;
        let score = cosine_similarity(query_embedding, &embedding);
        results.push(SearchResult {
            path: entry.path.clone(),
            title: entry.title.clone(),
            snippet: format!("Semantic match - {} tags", entry.tags.len()),
            score,
            result_type: "semantic".to_string(),
        });
    }
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(results.into_iter().take(10).collect())
}

async fn search_hnsw(base_dir: &Path, manifest: &EmbeddingManifest, query_embedding: &[f32]) -> Result<Vec<SearchResult>, String> {
    use usearch::{Index, ffi::{IndexOptions, MetricKind, ScalarKind}};
    let hnsw_path = base_dir.join(HNSW_FILE);
    
    let mut index = if tokio::fs::metadata(&hnsw_path).await.is_ok() {
        let options = IndexOptions {
            dimensions: EMBEDDING_DIM,
            metric: MetricKind::Cos,
            quantization: ScalarKind::F32,
            ..Default::default()
        };
        let index = Index::new(&options).map_err(|e| e.to_string())?;
        index.load(hnsw_path.to_str().unwrap()).map_err(|e| e.to_string())?;
        index
    } else {
        build_index_async(&hnsw_path, manifest, base_dir).await?
    };
    let matches = index.search(query_embedding, 10).map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    for (id, distance) in matches {
        if let Some(entry) = manifest.entries.iter().find(|entry| entry.id == id) {
            let score = 1.0 - distance;
            results.push(SearchResult {
                path: entry.path.clone(),
                title: entry.title.clone(),
                snippet: format!("Semantic match - {} tags", entry.tags.len()),
                score,
                result_type: "semantic".to_string(),
            });
        }
    }
    Ok(results)
}

async fn build_index_async(hnsw_path: &Path, manifest: &EmbeddingManifest, base_dir: &Path) -> Result<usearch::Index, String> {
    use usearch::{Index, ffi::{IndexOptions, MetricKind, ScalarKind}};
    let options = IndexOptions {
        dimensions: EMBEDDING_DIM,
        metric: MetricKind::Cos,
        quantization: ScalarKind::F32,
        ..Default::default()
    };
    let index = Index::new(&options).map_err(|e| e.to_string())?;
    for entry in &manifest.entries {
        let embedding_path = base_dir.join(&entry.embedding_file);
        let vector = read_embedding(&embedding_path).await?;
        index.add(entry.id, &vector).map_err(|e| e.to_string())?;
    }
    index.save(hnsw_path.to_str().unwrap()).map_err(|e| e.to_string())?;
    Ok(index)
}

async fn build_hnsw_index(base_dir: &Path, manifest: &EmbeddingManifest) -> Result<(), String> {
    let hnsw_path = base_dir.join(HNSW_FILE);
    if tokio::fs::metadata(&hnsw_path).await.is_ok() { let _ = tokio::fs::remove_file(&hnsw_path).await; }
    build_index_async(&hnsw_path, manifest, base_dir).await?;
    Ok(())
}

fn normalize(vector: &mut [f32]) {
    let norm: f32 = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 { for value in vector { *value /= norm; } }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0;
    for (x, y) in a.iter().zip(b.iter()) { sum += x * y; }
    sum
}

fn embeddings_dir(vault_path: &str) -> PathBuf { Path::new(vault_path).join(".quillpaw/embeddings") }
fn embedding_filename(path: &str) -> String { format!("{:x}.json", hash_id(path)) }
fn hash_id<T: Hash + ?Sized>(value: &T) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

async fn file_modified(path: &Path) -> Result<u64, String> {
    let metadata = tokio::fs::metadata(path).await.map_err(|e| e.to_string())?;
    let modified = metadata.modified().map_err(|e| e.to_string())?;
    Ok(modified.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())
}

fn should_skip(path: &Path) -> bool {
    path.components().any(|c| {
        c.as_os_str().to_string_lossy().starts_with(".quillpaw") || c.as_os_str().to_string_lossy().starts_with(".assets")
    })
}

fn parse_frontmatter(content: &str, path: &Path) -> (String, Vec<String>, String) {
    let mut title = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Untitled").to_string();
    let mut tags = vec![];
    let mut lines = content.lines();
    let first = lines.next().unwrap_or_default();
    if first.trim() == "---" {
        for line in &mut lines {
            if line.trim() == "---" { break; }
            if let Some((key, value)) = line.split_once(':') {
                let value = value.trim();
                match key.trim() {
                    "title" => title = value.trim_matches('"').to_string(),
                    "tags" => {
                        let trimmed = value.trim().trim_start_matches('[').trim_end_matches(']');
                        tags = trimmed.split(',').map(|item| item.trim().trim_matches('"').to_string()).filter(|item| !item.is_empty()).collect();
                    }
                    _ => {}
                }
            }
        }
    } else { lines = content.lines(); }
    let body = lines.collect::<Vec<_>>().join("\n");
    (title, tags, body)
}
