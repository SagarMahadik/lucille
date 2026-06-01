use lucille::{process_filter, process_filter_with_config, SearchNode, ParserConfig};
use serde_json::Value;

fn assert_filter_node(
    node: &SearchNode,
    expected_field: &str,
    expected_condition: &str,
    expected_value: Value,
    expected_field_type: &str,
    expected_custom: bool,
) {
    match node {
        SearchNode::Filter {
            field,
            condition,
            value,
            field_type,
            is_custom_property,
        } => {
            assert_eq!(field, expected_field, "field mismatch");
            assert_eq!(condition, expected_condition, "condition mismatch");
            assert_eq!(value, &expected_value, "value mismatch");
            assert_eq!(field_type, expected_field_type, "field_type mismatch");
            assert_eq!(is_custom_property, &expected_custom, "is_custom_property mismatch");
        }
        other => panic!("Expected Filter, got {:?}", other),
    }
}

// ── process_filter (uses defaults) ───────────────────────────

#[test]
fn test_process_filter_text_is() {
    let node = process_filter("domain", "google.com", false, false).unwrap();
    assert_filter_node(&node, "domain", "is", Value::String("google.com".to_string()), "text", false);
}

#[test]
fn test_process_filter_text_negated() {
    let node = process_filter("domain", "google.com", true, false).unwrap();
    assert_filter_node(&node, "domain", "isNot", Value::String("google.com".to_string()), "text", false);
}

#[test]
fn test_process_filter_boolean_true() {
    let node = process_filter("isFavorite", "true", false, false).unwrap();
    assert_filter_node(&node, "isFavorite", "true", Value::Bool(true), "boolean", false);
}

#[test]
fn test_process_filter_boolean_false() {
    let node = process_filter("isFavorite", "false", false, false).unwrap();
    assert_filter_node(&node, "isFavorite", "false", Value::Bool(false), "boolean", false);
}

#[test]
fn test_process_filter_boolean_negated_true() {
    let node = process_filter("isFavorite", "true", true, false).unwrap();
    assert_filter_node(&node, "isFavorite", "false", Value::Bool(false), "boolean", false);
}

#[test]
fn test_process_filter_boolean_negated_false() {
    let node = process_filter("isFavorite", "false", true, false).unwrap();
    assert_filter_node(&node, "isFavorite", "true", Value::Bool(true), "boolean", false);
}

#[test]
fn test_process_filter_number_equals() {
    let node = process_filter("rating", "7", false, false).unwrap();
    assert_filter_node(&node, "rating", "is", Value::from(7.0), "number", false);
}

#[test]
fn test_process_filter_number_greater_than() {
    let node = process_filter("rating", ">7", false, false).unwrap();
    assert_filter_node(&node, "rating", "greaterThan", Value::from(7.0), "number", false);
}

#[test]
fn test_process_filter_number_less_than() {
    let node = process_filter("rating", "<3", false, false).unwrap();
    assert_filter_node(&node, "rating", "lessThan", Value::from(3.0), "number", false);
}

#[test]
fn test_process_filter_number_gte() {
    let node = process_filter("rating", ">=5", false, false).unwrap();
    assert_filter_node(&node, "rating", "isGreaterThanOrEqualTo", Value::from(5.0), "number", false);
}

#[test]
fn test_process_filter_number_lte() {
    let node = process_filter("rating", "<=8", false, false).unwrap();
    assert_filter_node(&node, "rating", "isLessThanOrEqualTo", Value::from(8.0), "number", false);
}

#[test]
fn test_process_filter_number_negated_greater_than() {
    let node = process_filter("rating", ">7", true, false).unwrap();
    assert_filter_node(&node, "rating", "isLessThanOrEqualTo", Value::from(7.0), "number", false);
}

#[test]
fn test_process_filter_number_negated_less_than() {
    let node = process_filter("rating", "<3", true, false).unwrap();
    assert_filter_node(&node, "rating", "isGreaterThanOrEqualTo", Value::from(3.0), "number", false);
}

#[test]
fn test_process_filter_date_after() {
    let node = process_filter("createdAt", ">2024-01-01", false, false).unwrap();
    assert_filter_node(&node, "createdAt", "after", Value::String("2024-01-01".to_string()), "date", false);
}

#[test]
fn test_process_filter_date_before() {
    let node = process_filter("createdAt", "<2024-06-01", false, false).unwrap();
    assert_filter_node(&node, "createdAt", "before", Value::String("2024-06-01".to_string()), "date", false);
}

#[test]
fn test_process_filter_date_negated_after() {
    let node = process_filter("createdAt", ">2024-01-01", true, false).unwrap();
    assert_filter_node(&node, "createdAt", "onOrBefore", Value::String("2024-01-01".to_string()), "date", false);
}

#[test]
fn test_process_filter_tag() {
    let node = process_filter("tags", "rust", false, true).unwrap();
    assert_filter_node(&node, "tags", "includes", Value::String("rust".to_string()), "multiSelect", false);
}

#[test]
fn test_process_filter_tag_negated() {
    let node = process_filter("tags", "rust", true, true).unwrap();
    assert_filter_node(&node, "tags", "doesNotInclude", Value::String("rust".to_string()), "multiSelect", false);
}

#[test]
fn test_process_filter_multi_select_single() {
    let node = process_filter("tags", "rust", false, false).unwrap();
    assert_filter_node(&node, "tags", "includes", Value::String("rust".to_string()), "multiSelect", false);
}

#[test]
fn test_process_filter_multi_select_multiple() {
    let node = process_filter("tags", "rust,ts", false, false).unwrap();
    assert_filter_node(&node, "tags", "includeAnyOf", Value::Array(vec![
        Value::String("rust".to_string()),
        Value::String("ts".to_string()),
    ]), "multiSelect", false);
}

#[test]
fn test_process_filter_multi_select_negated_multiple() {
    let node = process_filter("tags", "rust,ts", true, false).unwrap();
    assert_filter_node(&node, "tags", "excludeAnyOf", Value::Array(vec![
        Value::String("rust".to_string()),
        Value::String("ts".to_string()),
    ]), "multiSelect", false);
}

#[test]
fn test_process_filter_text_wildcard_contains() {
    let node = process_filter("domain", "*google*", false, false).unwrap();
    assert_filter_node(&node, "domain", "contains", Value::String("google".to_string()), "text", false);
}

#[test]
fn test_process_filter_text_wildcard_starts_with() {
    let node = process_filter("domain", "google*", false, false).unwrap();
    assert_filter_node(&node, "domain", "startsWith", Value::String("google".to_string()), "text", false);
}

#[test]
fn test_process_filter_text_wildcard_ends_with() {
    let node = process_filter("domain", "*.com", false, false).unwrap();
    assert_filter_node(&node, "domain", "endsWith", Value::String(".com".to_string()), "text", false);
}

#[test]
fn test_process_filter_unknown_field_defaults_to_text() {
    let node = process_filter("unknownField", "hello", false, false).unwrap();
    assert_filter_node(&node, "unknownField", "is", Value::String("hello".to_string()), "text", false);
}

#[test]
fn test_process_filter_custom_property() {
    let node = process_filter("cp_author", "Taleb", false, false).unwrap();
    // hybrid_tokenize lowercases
    assert_filter_node(&node, "author", "is", Value::String("taleb".to_string()), "text", true);
}

#[test]
fn test_process_filter_custom_property_negated() {
    let node = process_filter("cp_author", "Taleb", true, false).unwrap();
    assert_filter_node(&node, "author", "isNot", Value::String("taleb".to_string()), "text", true);
}

#[test]
fn test_process_filter_text_comma_separated() {
    // Non-fts text field with commas
    let node = process_filter("domain", "google,example", false, false).unwrap();
    assert_filter_node(&node, "domain", "isAnyOf", Value::Array(vec![
        Value::String("google".to_string()),
        Value::String("example".to_string()),
    ]), "text", false);
}

#[test]
fn test_process_filter_text_fts_field_with_comma() {
    // FTS fields with commas don't split — they stay as a single string
    let node = process_filter("title", "hello,world", false, false).unwrap();
    assert_filter_node(&node, "title_tokens", "is", Value::String("hello world".to_string()), "text", false);
}

// ── process_filter_with_config (custom config) ───────────────

#[test]
fn test_process_filter_with_config_custom_field_type() {
    let config = ParserConfig::new().with_field("priority", "number");
    let node = process_filter_with_config("priority", ">3", false, false, &config).unwrap();
    assert_filter_node(&node, "priority", "greaterThan", Value::from(3.0), "number", false);
}

#[test]
fn test_process_filter_with_config_custom_abbreviation() {
    let config = ParserConfig::new()
        .with_field("language", "text")
        .with_abbreviation("lang", "language");
    let node = process_filter_with_config("lang", "rust", false, false, &config).unwrap();
    assert_filter_node(&node, "language", "is", Value::String("rust".to_string()), "text", false);
}

#[test]
fn test_process_filter_with_config_empty_config() {
    let config = ParserConfig::new();
    let node = process_filter_with_config("isFavorite", "true", false, false, &config).unwrap();
    // Unknown field → text (not boolean)
    assert_filter_node(&node, "isFavorite", "is", Value::String("true".to_string()), "text", false);
}

#[test]
fn test_process_filter_with_config_negation_and_abbreviation() {
    let config = ParserConfig::new()
        .with_field("rating", "number")
        .with_abbreviation("r", "rating");
    let node = process_filter_with_config("r", ">7", true, false, &config).unwrap();
    assert_filter_node(&node, "rating", "isLessThanOrEqualTo", Value::from(7.0), "number", false);
}
