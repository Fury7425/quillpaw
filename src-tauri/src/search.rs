use crate::models::SearchResult;

pub async fn build_index(_vault_path: &str) -> Result<(), String> {
    Ok(())
}

pub async fn keyword_search(_vault_path: &str, query: &str) -> Result<Vec<SearchResult>, String> {
    Ok(vec![SearchResult {
        path: String::new(),
        title: "Keyword Search".into(),
        snippet: format!("Placeholder result for: {query}"),
        score: 1.0,
        result_type: "keyword".into(),
    }])
}

pub async fn semantic_search(_vault_path: &str, query: &str) -> Result<Vec<SearchResult>, String> {
    Ok(vec![SearchResult {
        path: String::new(),
        title: "Semantic Search".into(),
        snippet: format!("Placeholder result for: {query}"),
        score: 0.9,
        result_type: "semantic".into(),
    }])
}
