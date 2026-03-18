use std::path::Path;

use tantivy::collector::TopDocs;
use tantivy::schema::{Field, Schema, SchemaBuilder, Value, STORED, STRING, TEXT};
use tantivy::snippet::SnippetGenerator;
use tantivy::{doc, Index};
use walkdir::WalkDir;

use crate::{embeddings, models::SearchResult};

/// Build a keyword search index for the vault.
pub async fn build_index(vault_path: &str) -> Result<(), String> {
    let vault_path = vault_path.to_string();
    tokio::task::spawn_blocking(move || {
        let index = open_or_create_index(&vault_path)?;
        let mut writer = index.writer(50_000_000).map_err(|e| e.to_string())?;
        writer.delete_all_documents().map_err(|e| e.to_string())?;
        let (_, fields) = schema_fields();
        for entry in WalkDir::new(&vault_path).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if should_skip_path(path) || path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
            let (meta, body) = parse_frontmatter(&content);
            let title = meta
                .title
                .unwrap_or_else(|| title_from_path(path).to_string());
            let mut all_tags = meta.tags;
            all_tags.extend(meta.aliases);
            let tags = all_tags.join(" ");
            writer
                .add_document(doc!(
                    fields.path => path.to_string_lossy().to_string(),
                    fields.title => title,
                    fields.body => body,
                    fields.tags => tags
                ))
                .map_err(|e| e.to_string())?;
        }
        writer.commit().map_err(|e| e.to_string())?;
        writer.wait_merging_threads().map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Run a keyword search query against the vault index.
pub async fn keyword_search(vault_path: &str, query: &str) -> Result<Vec<SearchResult>, String> {
    let vault = vault_path.to_string();
    let query = query.to_string();
    tokio::task::spawn_blocking(move || {
        let index = open_or_create_index(&vault)?;
        let reader = index.reader().map_err(|e| e.to_string())?;
        let searcher = reader.searcher();
        let (_schema, fields) = schema_fields();
        let parser = tantivy::query::QueryParser::for_index(
            &index,
            vec![fields.title, fields.body, fields.tags],
        );
        let query = parser.parse_query(&query).map_err(|e| e.to_string())?;
        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(25))
            .map_err(|e| e.to_string())?;
        let snippet_gen =
            SnippetGenerator::create(&searcher, &query, fields.body).map_err(|e| e.to_string())?;
        let mut results = vec![];
        for (score, doc_address) in top_docs {
            let retrieved: tantivy::TantivyDocument =
                searcher.doc(doc_address).map_err(|e| e.to_string())?;
            let path = retrieved
                .get_first(fields.path)
                .and_then(|v: &tantivy::schema::OwnedValue| v.as_str())
                .unwrap_or("")
                .to_string();
            let title = retrieved
                .get_first(fields.title)
                .and_then(|v: &tantivy::schema::OwnedValue| v.as_str())
                .unwrap_or("Untitled")
                .to_string();
            let snippet = snippet_gen.snippet_from_doc(&retrieved).to_html();
            results.push(SearchResult {
                path,
                title,
                snippet,
                score,
                result_type: "keyword".to_string(),
            });
        }
        Ok(results)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Placeholder semantic search until embeddings are available.
pub async fn semantic_search(vault_path: &str, query: &str) -> Result<Vec<SearchResult>, String> {
    embeddings::semantic_search(vault_path, query).await
}

#[derive(Clone)]
struct SearchFields {
    path: Field,
    title: Field,
    body: Field,
    tags: Field,
}

#[derive(Default)]
struct Frontmatter {
    title: Option<String>,
    tags: Vec<String>,
    aliases: Vec<String>,
}

fn schema_fields() -> (Schema, SearchFields) {
    let mut builder = SchemaBuilder::default();
    let path = builder.add_text_field("path", STRING | STORED);
    let title = builder.add_text_field("title", TEXT | STORED);
    let body = builder.add_text_field("body", TEXT | STORED);
    let tags = builder.add_text_field("tags", TEXT | STORED);
    let schema = builder.build();
    (
        schema,
        SearchFields {
            path,
            title,
            body,
            tags,
        },
    )
}

fn open_or_create_index(vault_path: &str) -> Result<Index, String> {
    let (schema, _) = schema_fields();
    let index_dir = Path::new(vault_path).join(".quillpaw/index");
    std::fs::create_dir_all(&index_dir).map_err(|e| e.to_string())?;
    if let Ok(index) = Index::open_in_dir(&index_dir) {
        Ok(index)
    } else {
        Index::create_in_dir(&index_dir, schema).map_err(|e| e.to_string())
    }
}

fn should_skip_path(path: &Path) -> bool {
    path.components().any(|c| {
        c.as_os_str().to_string_lossy().starts_with(".quillpaw")
            || c.as_os_str().to_string_lossy().starts_with(".assets")
    })
}

fn parse_frontmatter(content: &str) -> (Frontmatter, String) {
    let mut meta = Frontmatter::default();
    let mut lines = content.lines();
    let first = lines.next().unwrap_or_default();
    if first.trim() != "---" {
        return (meta, content.to_string());
    }
    let mut meta_lines = vec![];
    for line in &mut lines {
        if line.trim() == "---" {
            break;
        }
        meta_lines.push(line);
    }
    for line in meta_lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            let value = strip_quotes(value.trim());
            match key.trim() {
                "title" => meta.title = Some(value),
                "tags" => meta.tags = parse_list_value(&value),
                "aliases" => meta.aliases = parse_list_value(&value),
                _ => {}
            }
        }
    }
    let body = lines.collect::<Vec<_>>().join("\n");
    (meta, body)
}

fn strip_quotes(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

fn parse_list_value(value: &str) -> Vec<String> {
    let trimmed = value.trim();
    let content = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
    if content.is_empty() {
        return vec![];
    }
    content
        .split(',')
        .map(|item| strip_quotes(item.trim()))
        .filter(|item| !item.is_empty())
        .collect()
}

fn title_from_path(path: &Path) -> &str {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
}
