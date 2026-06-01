use lucille::{convert_to_structured, parse_query, StructuredQuery};
use serde_json::Value;

fn assert_filter(
    sq: &StructuredQuery,
    expected_field: &str,
    expected_condition: &str,
    expected_field_type: &str,
) {
    match sq {
        StructuredQuery::Filter {
            field,
            condition,
            field_type,
            ..
        } => {
            assert_eq!(field, expected_field, "field mismatch");
            assert_eq!(condition, expected_condition, "condition mismatch");
            assert_eq!(field_type, expected_field_type, "field_type mismatch");
        }
        other => panic!("Expected Filter, got {:?}", other),
    }
}

fn assert_filter_value(sq: &StructuredQuery, expected_value: Value) {
    match sq {
        StructuredQuery::Filter { value, .. } => {
            assert_eq!(value, &expected_value, "value mismatch");
        }
        other => panic!("Expected Filter, got {:?}", other),
    }
}

fn assert_group(sq: &StructuredQuery, expected_operator: &str, expected_count: usize) {
    match sq {
        StructuredQuery::Group { operator, filters } => {
            assert_eq!(operator, expected_operator, "operator mismatch");
            assert_eq!(filters.len(), expected_count, "filter count mismatch");
        }
        other => panic!("Expected Group, got {:?}", other),
    }
}

// ── Parse error / edge cases ─────────────────────────────────

#[test]
fn test_unclosed_quote_parses_as_content() {
    // Unclosed quote should still parse without panic
    let sq = parse_query("title:\"hello").and_then(|n| convert_to_structured(&n));
    assert!(sq.is_some(), "unclosed quote should still produce output");
}

#[test]
fn test_empty_parentheses() {
    let sq = parse_query("()").and_then(|n| convert_to_structured(&n));
    assert!(sq.is_none(), "empty parentheses should yield None");
}

#[test]
fn test_parentheses_with_only_whitespace() {
    let sq = parse_query("(  )").and_then(|n| convert_to_structured(&n));
    assert!(sq.is_none(), "whitespace-only parentheses should yield None");
}

#[test]
fn test_consecutive_not_does_not_panic() {
    // The parser does not support consecutive NOT operators;
    // this just checks it doesn't panic.
    let result1 = parse_query("NOT NOT domain:google.com").and_then(|n| convert_to_structured(&n));
    let result2 = parse_query("NOT NOT NOT domain:google.com").and_then(|n| convert_to_structured(&n));
    // May produce Some or None depending on implementation — just don't panic
    let _ = (result1, result2);
}

#[test]
fn test_contradictory_filters() {
    // Two filters with opposite values — they parse fine, just contradictory
    let sq = parse_query("isFavorite:true -isFavorite:true")
        .and_then(|n| convert_to_structured(&n))
        .expect("contradictory should parse");
    assert_group(&sq, "and", 2);
    if let StructuredQuery::Group { filters, .. } = &sq {
        assert_filter(&filters[0], "isFavorite", "true", "boolean");
        assert_filter(&filters[1], "isFavorite", "false", "boolean");
    }
}

#[test]
fn test_double_colon_parses() {
    let sq = parse_query("domain::google.com").and_then(|n| convert_to_structured(&n));
    assert!(sq.is_some(), "double colon should parse");
}

#[test]
fn test_trailing_colon() {
    let sq = parse_query("type:").and_then(|n| convert_to_structured(&n));
    assert!(sq.is_some(), "trailing colon should parse");
    if let Some(sq) = &sq {
        assert_filter(sq, "type", "is", "text");
        assert_filter_value(sq, Value::String("".to_string()));
    }
}

#[test]
fn test_field_name_with_hyphen_preserved() {
    let sq = parse_query("my-field:value").and_then(|n| convert_to_structured(&n)).unwrap();
    // Not a known field, defaults to text
    assert_filter(&sq, "my-field", "is", "text");
}

#[test]
fn test_not_at_end_of_query() {
    // NOT at end becomes content text; both terms are pure content,
    // so AND-to-OR conversion kicks in
    let sq = parse_query("hello NOT").and_then(|n| convert_to_structured(&n)).unwrap();
    match &sq {
        StructuredQuery::Group { operator, filters } => {
            assert_eq!(operator, "or");
            assert_eq!(filters.len(), 2);
        }
        StructuredQuery::Filter { .. } => {} // single filter also fine
        other => panic!("Unexpected shape: {:?}", other),
    }
}

#[test]
fn test_malformed_number_operator() {
    // > without number after should still parse as text
    let sq = parse_query("rating:>").and_then(|n| convert_to_structured(&n)).unwrap();
    // ">" is not a valid number, so it stays as text/is condition
    assert_filter(&sq, "rating", "is", "number");
}

#[test]
fn test_number_with_invalid_operator() {
    // != is not a recognized operator
    let sq = parse_query("rating:!=5").and_then(|n| convert_to_structured(&n)).unwrap();
    // Falls through to text handling...
    assert_filter(&sq, "rating", "is", "number");
}

#[test]
fn test_unclosed_parens_query() {
    let sq = parse_query("(domain:google.com").and_then(|n| convert_to_structured(&n));
    assert!(sq.is_some(), "unclosed parens should still parse");
    if let Some(sq) = &sq {
        assert_filter(sq, "domain", "is", "text");
    }
}

#[test]
fn test_extra_closing_parens() {
    let sq = parse_query("domain:google.com)").and_then(|n| convert_to_structured(&n));
    assert!(sq.is_some(), "extra closing paren should still parse");
}

#[test]
fn test_starts_with_operator_not_wildcard() {
    // "^" is not a recognized operator — should be treated as literal
    let sq = parse_query("title:^hello").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "title_tokens", "is", "text");
    assert_filter_value(&sq, Value::String("hello".to_string()));
}

#[test]
fn test_query_with_only_not_keyword() {
    // "NOT" by itself should parse as content
    let result = parse_query("NOT").and_then(|n| convert_to_structured(&n));
    // Depending on implementation, could be Some or None
    // Just make sure it doesn't panic
    let _ = result;
}

#[test]
fn test_very_long_query() {
    let long_val = "x".repeat(1000);
    let query = format!("title:{}", long_val);
    let sq = parse_query(&query).and_then(|n| convert_to_structured(&n));
    assert!(sq.is_some(), "long query should parse without panic");
}

#[test]
fn test_single_pipe_character() {
    // Pipe is in allowed chars
    let sq = parse_query("a|b").and_then(|n| convert_to_structured(&n)).unwrap();
    match &sq {
        StructuredQuery::Filter { value, .. } => {
            // "a|b" -> hybrid_tokenize("a|b") -> "a b" (pipe is not in [a-z0-9._-])
            assert_eq!(value.as_str().unwrap(), "a b");
        }
        _ => panic!("Expected Filter"),
    }
}

#[test]
fn test_hash_in_value_triggers_tag() {
    // # in the query triggers tag parsing; "type:C" becomes one filter
    // and "#" becomes an (empty) tag
    let sq = parse_query("type:C#").and_then(|n| convert_to_structured(&n)).unwrap();
    match &sq {
        StructuredQuery::Group { operator, filters } => {
            assert_eq!(operator, "and");
            assert_eq!(filters.len(), 2);
        }
        _ => panic!("Expected Group"),
    }
}
