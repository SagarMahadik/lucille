use lucille::{convert_to_structured, extract_note_text, hybrid_tokenize, normalize_for_search, parse_query, StructuredQuery};
use serde_json::json;

// ── normalize_for_search ─────────────────────────────────────

#[test]
fn test_normalize_lowercases() {
    assert_eq!(normalize_for_search("Hello World"), "hello world");
}

#[test]
fn test_normalize_already_lower() {
    assert_eq!(normalize_for_search("hello world"), "hello world");
}

#[test]
fn test_normalize_empty() {
    assert_eq!(normalize_for_search(""), "");
}

#[test]
fn test_normalize_mixed_case() {
    assert_eq!(normalize_for_search("RustLang"), "rustlang");
}

#[test]
fn test_normalize_numbers_unchanged() {
    assert_eq!(normalize_for_search("123"), "123");
}

#[test]
fn test_normalize_special_chars_unchanged() {
    assert_eq!(normalize_for_search("hello-world"), "hello-world");
}

// ── hybrid_tokenize ──────────────────────────────────────────

#[test]
fn test_hybrid_tokenize_simple() {
    assert_eq!(hybrid_tokenize("hello world"), "hello world");
}

#[test]
fn test_hybrid_tokenize_empty() {
    assert_eq!(hybrid_tokenize(""), "");
}

#[test]
fn test_hybrid_tokenize_single_word() {
    assert_eq!(hybrid_tokenize("rust"), "rust");
}

#[test]
fn test_hybrid_tokenize_strips_punctuation() {
    assert_eq!(hybrid_tokenize("hello, world!"), "hello world");
}

#[test]
fn test_hybrid_tokenize_preserves_dots_hyphens_underscores() {
    assert_eq!(hybrid_tokenize("hello-world.com/file_name"), "hello-world.com file_name");
}

#[test]
fn test_hybrid_tokenize_strips_slash() {
    assert_eq!(hybrid_tokenize("a/b"), "a b");
}

#[test]
fn test_hybrid_tokenize_normalizes_case() {
    assert_eq!(hybrid_tokenize("Hello World"), "hello world");
}

#[test]
fn test_hybrid_tokenize_multiple_spaces() {
    assert_eq!(hybrid_tokenize("hello   world"), "hello world");
}

#[test]
fn test_hybrid_tokenize_leading_trailing_whitespace() {
    assert_eq!(hybrid_tokenize("  hello world  "), "hello world");
}

#[test]
fn test_hybrid_tokenize_only_special_chars() {
    assert_eq!(hybrid_tokenize("@#$%"), "");
}

#[test]
fn test_hybrid_tokenize_symbols_surrounding_words() {
    assert_eq!(hybrid_tokenize("(hello) [world] {foo}"), "hello world foo");
}

#[test]
fn test_hybrid_tokenize_colon_and_semicolon() {
    assert_eq!(hybrid_tokenize("a:b;c"), "a b c");
}

#[test]
fn test_hybrid_tokenize_keeps_underscore_separated() {
    assert_eq!(hybrid_tokenize("hello_world"), "hello_world");
}

// ── extract_note_text ────────────────────────────────────────

#[test]
fn test_extract_note_text_simple_paragraph() {
    let note = json!({
        "type": "doc",
        "content": [
            {
                "type": "paragraph",
                "content": [
                    { "text": "Hello, this is a note", "type": "text" }
                ]
            }
        ]
    });
    let result = extract_note_text(&note);
    assert_eq!(result, "Hello, this is a note");
}

#[test]
fn test_extract_note_text_multiple_paragraphs() {
    let note = json!({
        "type": "doc",
        "content": [
            {
                "type": "paragraph",
                "content": [
                    { "text": "First para", "type": "text" }
                ]
            },
            {
                "type": "paragraph",
                "content": [
                    { "text": "Second para", "type": "text" }
                ]
            }
        ]
    });
    let result = extract_note_text(&note);
    assert_eq!(result, "First para Second para");
}

#[test]
fn test_extract_note_text_with_headings() {
    let note = json!({
        "type": "doc",
        "content": [
            {
                "type": "heading",
                "content": [
                    { "text": "Title", "type": "text" }
                ]
            },
            {
                "type": "paragraph",
                "content": [
                    { "text": "Body", "type": "text" }
                ]
            }
        ]
    });
    let result = extract_note_text(&note);
    assert_eq!(result, "Title Body");
}

#[test]
fn test_extract_note_text_empty() {
    let result = extract_note_text(&json!("not an object"));
    assert_eq!(result, "");
}

#[test]
fn test_extract_note_text_empty_object() {
    let result = extract_note_text(&json!({}));
    assert_eq!(result, "");
}

#[test]
fn test_extract_note_text_no_content_array() {
    let note = json!({ "type": "doc" });
    let result = extract_note_text(&note);
    assert_eq!(result, "");
}

#[test]
fn test_extract_note_text_empty_content() {
    let note = json!({ "type": "doc", "content": [] });
    let result = extract_note_text(&note);
    assert_eq!(result, "");
}

#[test]
fn test_extract_note_text_nested_formatting() {
    let note = json!({
        "type": "doc",
        "content": [
            {
                "type": "paragraph",
                "content": [
                    { "text": "Hello ", "type": "text" },
                    { "text": "bold", "type": "text", "marks": [{ "type": "bold" }] },
                    { "text": " world", "type": "text" }
                ]
            }
        ]
    });
    let result = extract_note_text(&note);
    assert_eq!(result, "Hello bold world");
}

#[test]
fn test_extract_note_text_whitespace_collapsed() {
    let note = json!({
        "type": "doc",
        "content": [
            {
                "type": "paragraph",
                "content": [
                    { "text": "Hello    world", "type": "text" }
                ]
            }
        ]
    });
    let result = extract_note_text(&note);
    assert_eq!(result, "Hello world");
}

// ── Unicode and emoji content search ─────────────────────────

// The tokenizer regex [^a-z0-9._-]+ strips non-ASCII characters.
// These tests document that limitation.

#[test]
fn test_unicode_content_search_strips_non_ascii() {
    // "é" is stripped by tokenizer, leaving just "caf"
    let sq = parse_query("café").and_then(|n| convert_to_structured(&n)).unwrap();
    match &sq {
        StructuredQuery::Filter { field, value, .. } => {
            assert_eq!(field, "content_tokens");
            assert_eq!(value.as_str().unwrap(), "caf");
        }
        _ => panic!("Expected Filter"),
    }
}

#[test]
fn test_unicode_field_value_strips_non_ascii() {
    let sq = parse_query("title:café").and_then(|n| convert_to_structured(&n)).unwrap();
    match &sq {
        StructuredQuery::Filter { field, value, .. } => {
            assert_eq!(field, "title_tokens");
            assert_eq!(value.as_str().unwrap(), "caf");
        }
        _ => panic!("Expected Filter"),
    }
}

#[test]
fn test_emoji_in_content_splits_tokens() {
    // Emoji acts as delimiter
    let sq = parse_query("hello🚀world").and_then(|n| convert_to_structured(&n)).unwrap();
    match &sq {
        StructuredQuery::Group { operator, filters } => {
            assert_eq!(operator, "or");
            assert_eq!(filters.len(), 2);
        }
        _ => panic!("Expected Group (OR)"),
    }
}

#[test]
fn test_cjk_characters_stripped_by_hybrid_tokenize() {
    // CJK chars are stripped by hybrid_tokenize — the regex only keeps [a-z0-9._-]
    let tok = hybrid_tokenize("你好世界");
    assert_eq!(tok, "", "CJK chars should be stripped by hybrid_tokenize");
    // But the parser keeps them; hybrid_tokenize is only applied after field resolution
    let sq = parse_query("你好").and_then(|n| convert_to_structured(&n)).unwrap();
    match &sq {
        StructuredQuery::Filter { field, value, .. } => {
            assert_eq!(field, "content_tokens");
            // hybrid_tokenize strips the CJK chars
            assert_eq!(value.as_str().unwrap(), "");
        }
        _ => panic!("Expected Filter"),
    }
}

#[test]
fn test_mixed_ascii_and_unicode() {
    // "东京" stripped, "-2024" keeps the hyphen and "2024"
    let sq = parse_query("东京-2024").and_then(|n| convert_to_structured(&n)).unwrap();
    match &sq {
        StructuredQuery::Filter { field, value, .. } => {
            assert_eq!(field, "content_tokens");
            assert_eq!(value.as_str().unwrap(), "-2024");
        }
        _ => panic!("Expected Filter"),
    }
}

#[test]
fn test_accented_characters_in_tag_preserved() {
    let sq = parse_query("#résumé").and_then(|n| convert_to_structured(&n)).unwrap();
    match &sq {
        StructuredQuery::Filter { field, value, condition, .. } => {
            assert_eq!(field, "tags");
            assert_eq!(condition, "includes");
            assert_eq!(value.as_str().unwrap(), "résumé");
        }
        _ => panic!("Expected Filter"),
    }
}
