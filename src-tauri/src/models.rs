use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_folder: bool,
    pub children: Option<Vec<FileNode>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NoteContent {
    pub path: String,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub created: String,
    pub modified: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchResult {
    pub path: String,
    pub title: String,
    pub snippet: String,
    pub score: f32,
    pub result_type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AiProposal {
    pub id: String,
    pub proposal_type: String,
    pub title: String,
    pub content: String,
    pub target_path: Option<String>,
    pub metadata: Option<serde_json::Value>,
}
