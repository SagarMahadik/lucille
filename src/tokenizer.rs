use serde_json::Value;
use regex::Regex;
use once_cell::sync::Lazy;

pub fn normalize_for_search(text: &str) -> String {
    text.to_lowercase()
}

pub fn hybrid_tokenize(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    let normalized = normalize_for_search(text);
    let mut tokens = Vec::new();

    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^a-z0-9._-]+").unwrap());

    for word in RE.split(&normalized) {
        if !word.is_empty() {
            tokens.push(word.to_string());
        }
    }

    tokens.join(" ")
}

pub fn extract_note_text(note_json: &Value) -> String {
    if !note_json.is_object() || note_json.get("content").map_or(true, |c| !c.is_array()) {
        return String::new();
    }

    let mut text_parts = Vec::new();
    traverse_note(note_json, &mut text_parts);

    let joined = text_parts.join("");
    static SPACE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
    SPACE_RE.replace_all(&joined, " ").trim().to_string()
}

fn traverse_note(node: &Value, text_parts: &mut Vec<String>) {
    if let Some(text) = node.get("text").and_then(|t| t.as_str()) {
        text_parts.push(text.to_string());
    }

    if let Some(content) = node.get("content").and_then(|c| c.as_array()) {
        for child in content {
            traverse_note(child, text_parts);
        }

        if let Some(node_type) = node.get("type").and_then(|t| t.as_str()) {
            if ["paragraph", "heading", "list_item"].contains(&node_type) {
                text_parts.push(" ".to_string());
            }
        }
    }
}
