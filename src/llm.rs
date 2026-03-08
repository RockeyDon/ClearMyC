use crate::data::{AppData, FolderEntry, LlmConfig};
use crate::file_ops::calculate_dir_size;
use std::collections::VecDeque;
use std::time::SystemTime;

/// Test LLM connection by sending a simple request to the OpenAI-compatible API.
pub async fn test_llm_connection(llm: LlmConfig) -> String {
    if llm.base_url.trim().is_empty() {
        return "Error: BaseUrl is required.".to_string();
    }

    let url = format!(
        "{}/chat/completions",
        llm.base_url.trim_end_matches('/')
    );

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    if !llm.api_key.is_empty() {
        if let Ok(val) = format!("Bearer {}", llm.api_key).parse() {
            headers.insert("Authorization", val);
        }
    }

    let model = if llm.model.is_empty() {
        "gpt-3.5-turbo".to_string()
    } else {
        llm.model.clone()
    };

    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": "Hi"}],
        "max_tokens": 5
    });

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(e) => return format!("Error creating HTTP client: {}", e),
    };

    match client.post(&url).headers(headers).json(&body).send().await {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                "Connection successful!".to_string()
            } else {
                let body_text = resp.text().await.unwrap_or_default();
                format!("Error {}: {}", status.as_u16(), body_text)
            }
        }
        Err(e) => {
            if e.is_timeout() {
                "Error: Connection timed out (15s).".to_string()
            } else if e.is_connect() {
                format!("Error: Could not connect to {}.", llm.base_url)
            } else {
                format!("Error: {}", e)
            }
        }
    }
}

/// Auto-discover directories using BFS from C:\, filtering by strategy,
/// then asking the LLM which ones are safe to delete.
pub async fn auto_discover(lang: String, data_dir: std::path::PathBuf) -> String {
    // Load fresh data from disk
    let data = AppData::load_from(data_dir);
    let llm = data.read_llm_config();
    let strategy = data.read_strategy_config();
    let folders = &data.folders;

    if llm.base_url.trim().is_empty() {
        return "Error: LLM BaseUrl is not configured. Please set it in Strategy settings.".to_string();
    }

    // Parse skip list
    let skip_paths: Vec<String> = strategy
        .skip_list
        .split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let existing_paths: Vec<String> = folders.iter().map(|f| f.path.clone()).collect();

    let min_size_bytes: u64 = strategy.min_size * 1024 * 1024; // min_size is in MB
    let skip_recent_days = strategy.skip_recent;

    // BFS from C:\
    let mut queue: VecDeque<String> = VecDeque::new();
    // We collect (path, size) for every directory that passes skip_list
    // and existing_paths filters.  The min_size and skip_recent filters
    // are applied *after* the full traversal.
    let mut candidates: Vec<(String, u64)> = Vec::new();

    // Seed with top-level directories under C:\
    if let Ok(entries) = std::fs::read_dir("C:\\") {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                queue.push_back(entry.path().to_string_lossy().to_string());
            }
        }
    }

    while let Some(dir_path) = queue.pop_front() {
        // Check skip_list
        let normalized = dir_path.trim_end_matches('\\').to_string();
        if skip_paths.iter().any(|s| {
            let s_norm = s.trim_end_matches('\\');
            s_norm.eq_ignore_ascii_case(&normalized)
        }) {
            continue; // skip this directory and all its children
        }

        // Check existing folders
        if existing_paths.iter().any(|p| {
            let p_norm = p.trim_end_matches('\\');
            p_norm.eq_ignore_ascii_case(&normalized)
        }) {
            continue; // skip this directory and all its children
        }

        // Compute size for this directory
        let size = calculate_dir_size(&dir_path);

        // If size < min_size, this directory (and all its children) are
        // too small to care about — skip entirely.
        if size < min_size_bytes {
            continue;
        }

        // Size >= min_size — record it as a candidate and continue BFS
        // into its children so we can decide later whether to keep the
        // parent or its qualifying children.
        candidates.push((dir_path.clone(), size));

        // Enqueue children for BFS
        if let Ok(entries) = std::fs::read_dir(&dir_path) {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    queue.push_back(entry.path().to_string_lossy().to_string());
                }
            }
        }
    }

    // ── Post-BFS: prune parents that have qualifying children ────────
    //
    // If a parent directory AND one or more of its children both appear
    // in `candidates`, we only keep the children (they are more specific).
    // A parent is kept only when none of its children qualified.
    //
    // Because candidates are in BFS order (parents before children), we
    // can build a set of "has qualifying child" parents efficiently.

    let candidate_set: std::collections::HashSet<String> = candidates
        .iter()
        .map(|(p, _)| p.trim_end_matches('\\').to_ascii_lowercase())
        .collect();

    let mut parents_with_children: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    for (path, _) in &candidates {
        // Walk up the path to mark every ancestor that is also a candidate
        let normalized = path.trim_end_matches('\\').to_string();
        let p = std::path::Path::new(&normalized);
        let mut ancestor = p.parent();
        while let Some(a) = ancestor {
            let a_lower = a.to_string_lossy().to_ascii_lowercase();
            if candidate_set.contains(&a_lower) {
                parents_with_children.insert(a_lower);
                break; // only the nearest ancestor matters for marking
            }
            ancestor = a.parent();
        }
    }

    // Keep only candidates that are NOT parents-with-children
    let after_prune: Vec<String> = candidates
        .into_iter()
        .filter(|(path, _)| {
            let key = path.trim_end_matches('\\').to_ascii_lowercase();
            !parents_with_children.contains(&key)
        })
        .map(|(path, _)| path)
        .collect();

    // ── Post-BFS: filter by skip_recent ──────────────────────────────
    let surviving: Vec<String> = if skip_recent_days > 0 {
        after_prune
            .into_iter()
            .filter(|dir_path| {
                if let Ok(metadata) = std::fs::metadata(dir_path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                            let days = elapsed.as_secs() / 86400;
                            return days >= skip_recent_days as u64;
                        }
                    }
                }
                true // if we can't read metadata, keep it
            })
            .collect()
    } else {
        after_prune
    };

    if surviving.is_empty() {
        return "No candidate directories found after filtering.".to_string();
    }

    // Build directory tree string for LLM
    let tree_str = surviving.join("\n");

    // Call LLM
    let prompt = format!(
        "Below is a list of directories on a Windows PC. Help me identify which directories are safe to delete (e.g. caches, temp files, old logs, build artifacts). Return ONLY a JSON array in this exact format, no other text:\n[{{\"path\": \"folder path\", \"description\": \"brief reason(reason must be in {})\"}}]\n\nDirectories:\n{}",
        lang,
        tree_str
    );

    let url = format!(
        "{}/chat/completions",
        llm.base_url.trim_end_matches('/')
    );

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    if !llm.api_key.is_empty() {
        if let Ok(val) = format!("Bearer {}", llm.api_key).parse() {
            headers.insert("Authorization", val);
        }
    }

    let model = if llm.model.is_empty() {
        "gpt-3.5-turbo".to_string()
    } else {
        llm.model.clone()
    };

    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.2
    });

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
    {
        Ok(c) => c,
        Err(e) => return format!("Error creating HTTP client: {}", e),
    };

    let response = match client.post(&url).headers(headers).json(&body).send().await {
        Ok(resp) => {
            let status = resp.status();
            if !status.is_success() {
                let body_text = resp.text().await.unwrap_or_default();
                return format!("LLM API Error {}: {}", status.as_u16(), body_text);
            }
            match resp.text().await {
                Ok(t) => t,
                Err(e) => return format!("Error reading LLM response: {}", e),
            }
        }
        Err(e) => return format!("Error calling LLM: {}", e),
    };

    // Parse the LLM response to extract the JSON array
    let parsed: serde_json::Value = match serde_json::from_str(&response) {
        Ok(v) => v,
        Err(e) => return format!("Error parsing LLM response JSON: {}", e),
    };

    // Extract content from OpenAI-style response
    let content = match parsed["choices"][0]["message"]["content"].as_str() {
        Some(c) => c.to_string(),
        None => return format!("Unexpected LLM response format: {}", response),
    };

    // The content should be a JSON array; try to extract it
    let json_str = extract_json_array(&content);

    let discovered: Vec<serde_json::Value> = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => {
            return format!(
                "Error parsing LLM suggestions: {}. Raw content: {}",
                e, content
            );
        }
    };

    if discovered.is_empty() {
        return "LLM found no directories to suggest for deletion.".to_string();
    }

    // Convert to FolderEntry: LLM-selected ones get selected=true,
    // surviving ones not selected by LLM get selected=false.
    let selected_paths: std::collections::HashSet<String> = discovered
        .iter()
        .filter_map(|item| {
            item["path"].as_str().map(|p| p.trim_end_matches('\\').to_ascii_lowercase())
        })
        .collect();

    let mut new_folders: Vec<FolderEntry> = Vec::new();

    // First add LLM-selected entries (with description)
    for item in &discovered {
        if let Some(path) = item["path"].as_str() {
            if path.is_empty() {
                continue;
            }
            let description = item["description"].as_str().unwrap_or("").to_string();
            new_folders.push(FolderEntry {
                path: path.to_string(),
                description,
                selected: true,
            });
        }
    }

    // Then add surviving entries not selected by LLM
    for path in &surviving {
        let normalized = path.trim_end_matches('\\').to_ascii_lowercase();
        if !selected_paths.contains(&normalized) {
            new_folders.push(FolderEntry {
                path: path.clone(),
                description: String::new(),
                selected: false,
            });
        }
    }

    let selected_count = selected_paths.len();
    let total_count = new_folders.len();

    // Reload data and append
    let mut fresh_data = AppData::load_from(data.data_dir.clone());
    fresh_data.append_folders(new_folders);

    format!(
        "Auto Discovery complete! Found {} directories ({} suggested for cleanup, {} kept unchecked).",
        total_count, selected_count, total_count - selected_count
    )
}

/// Extract a JSON array from a string that might contain markdown fences or extra text.
fn extract_json_array(s: &str) -> String {
    // Try to find content between ```json ... ``` or ``` ... ```
    let s = s.trim();
    if let Some(start) = s.find('[') {
        if let Some(end) = s.rfind(']') {
            return s[start..=end].to_string();
        }
    }
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_array_plain() {
        let input = r#"[{"path": "C:\\test", "description": "cache"}]"#;
        let result = extract_json_array(input);
        assert!(result.starts_with('['));
        assert!(result.ends_with(']'));
    }

    #[test]
    fn test_extract_json_array_with_markdown() {
        let input = "Here are the results:\n```json\n[{\"path\": \"C:\\\\test\"}]\n```\n";
        let result = extract_json_array(input);
        assert!(result.starts_with('['));
        assert!(result.ends_with(']'));
    }

    #[test]
    fn test_extract_json_array_with_extra_text() {
        let input = "Sure! Here is the list:\n[{\"path\": \"C:\\\\temp\", \"description\": \"temp\"}]\nLet me know if you need more.";
        let result = extract_json_array(input);
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.len(), 1);
    }

    #[test]
    fn test_extract_json_array_no_brackets() {
        let input = "No JSON here at all";
        let result = extract_json_array(input);
        assert_eq!(result, input); // returns as-is
    }

    #[test]
    fn test_extract_json_array_empty_array() {
        let input = "[]";
        let result = extract_json_array(input);
        assert_eq!(result, "[]");
    }

    #[tokio::test]
    async fn test_llm_connection_empty_base_url() {
        let llm = LlmConfig {
            base_url: "".to_string(),
            model: "test".to_string(),
            api_key: "key".to_string(),
        };
        let result = test_llm_connection(llm).await;
        assert!(result.contains("BaseUrl is required"));
    }

    #[tokio::test]
    async fn test_llm_connection_whitespace_base_url() {
        let llm = LlmConfig {
            base_url: "   ".to_string(),
            model: "test".to_string(),
            api_key: "key".to_string(),
        };
        let result = test_llm_connection(llm).await;
        assert!(result.contains("BaseUrl is required"));
    }

    // #[tokio::test]
    // async fn test_llm_connection_unreachable_host() {
    //     let llm = LlmConfig {
    //         base_url: "http://192.0.2.1:1".to_string(), // RFC 5737 TEST-NET, unreachable
    //         model: "test".to_string(),
    //         api_key: "".to_string(),
    //     };
    //     let result = test_llm_connection(llm).await;
    //     assert!(result.contains("Error"));
    // }

    #[tokio::test]
    async fn test_auto_discover_no_base_url() {
        let dir = std::env::temp_dir().join(format!("clearmyc_llm_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let mut data = AppData::load_from(dir.clone());
        // Leave LLM base_url empty
        data.update_llm_config(String::new(), String::new(), String::new());

        let result = auto_discover(String::from("en-US"), dir.clone()).await;
        assert!(result.contains("LLM BaseUrl is not configured"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
