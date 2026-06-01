use lucille::{convert_to_structured, parse_query, parse_query_with_config, ParserConfig, StructuredQuery};
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

#[test]
fn test_custom_field_type_via_config() {
    let config = ParserConfig::new()
        .with_field("price", "number");
    let sq = parse_query_with_config("price:>100", &config)
        .and_then(|n| convert_to_structured(&n))
        .expect("should parse with custom config");
    assert_filter(&sq, "price", "greaterThan", "number");
    assert_filter_value(&sq, Value::from(100.0));
}

#[test]
fn test_custom_abbreviation_via_config() {
    let config = ParserConfig::new()
        .with_field("lang", "text") // so it's a known field, not defaulting to content_tokens
        .with_abbreviation("l", "lang");
    let sq = parse_query_with_config("l:rust", &config)
        .and_then(|n| convert_to_structured(&n))
        .expect("should parse with custom abbreviation");

    assert_filter(&sq, "lang", "is", "text");
    assert_filter_value(&sq, Value::String("rust".to_string()));
}

#[test]
fn test_empty_config_uses_default() {
    // Empty config with no field types — everything is text
    let config = ParserConfig::new();
    let sq = parse_query_with_config("isFavorite:true", &config)
        .and_then(|n| convert_to_structured(&n))
        .expect("should parse with empty config");
    // Since isFavorite is unknown (no field_types), it falls back to "text"
    // so the value is string "true" not bool true
    assert_filter(&sq, "isFavorite", "is", "text");
    assert_filter_value(&sq, Value::String("true".to_string()));
}

#[test]
fn test_custom_config_ignores_cherrypic_defaults() {
    let config = ParserConfig::new()
        .with_field("stars", "number");
    let sq = parse_query_with_config("stars:5", &config)
        .and_then(|n| convert_to_structured(&n))
        .expect("should parse");
    assert_filter(&sq, "stars", "is", "number");
    assert_filter_value(&sq, Value::from(5.0));

    // tags should NOT be multiSelect because we didn't include it
    let sq2 = parse_query_with_config("tags:rust", &config)
        .and_then(|n| convert_to_structured(&n))
        .expect("should parse");
    assert_filter(&sq2, "tags", "is", "text");
}

#[test]
fn test_abbreviation_prevails_over_builtin() {
    // With empty config, our abbreviation maps "t" to "type"
    let config = ParserConfig::new()
        .with_field("type", "text")
        .with_abbreviation("t", "type");
    let sq = parse_query_with_config("t:article", &config)
        .and_then(|n| convert_to_structured(&n))
        .expect("should parse");
    assert_filter(&sq, "type", "is", "text");
    assert_filter_value(&sq, Value::String("article".to_string()));
}

#[test]
fn test_cherrypic_defaults_matches_parse_query() {
    let config = ParserConfig::cherrypic_defaults();
    let query = "isFavorite:true domain:google.com";
    let default_sq = parse_query(query).and_then(|n| convert_to_structured(&n));
    let config_sq = parse_query_with_config(query, &config).and_then(|n| convert_to_structured(&n));
    assert_eq!(default_sq, config_sq, "cherrypic_defaults should match parse_query");
}

#[test]
fn test_process_filter_with_custom_config() {
    let config = ParserConfig::new()
        .with_field("priority", "number")
        .with_abbreviation("p", "priority");
    use lucille::process_filter_with_config;
    let node = process_filter_with_config("p", ">3", false, false, &config)
        .expect("should create filter");
    if let lucille::SearchNode::Filter { field, condition, value, field_type, .. } = &node {
        assert_eq!(field, "priority");
        assert_eq!(condition, "greaterThan");
        assert_eq!(value.as_f64(), Some(3.0));
        assert_eq!(field_type, "number");
    } else {
        panic!("Expected Filter node");
    }
}

#[test]
fn test_parse_query_with_config_no_abbreviations() {
    let config = ParserConfig::new()
        .with_field("createdAt", "date");
    // Without abbreviation "created" -> "createdAt", the unknown field becomes text.
    let sq = parse_query_with_config("created:>2024-01-01", &config)
        .and_then(|n| convert_to_structured(&n))
        .expect("should parse");
    // Unknown field defaults to text; ">" is not a wildcard, so condition is "is"
    assert_filter(&sq, "created", "is", "text");
    assert_filter_value(&sq, Value::String("2024-01-01".to_string()));
}

#[test]
fn test_custom_config_with_boolean_field() {
    let config = ParserConfig::new()
        .with_field("isActive", "boolean");
    let sq = parse_query_with_config("isActive:true", &config)
        .and_then(|n| convert_to_structured(&n))
        .expect("should parse");
    assert_filter(&sq, "isActive", "true", "boolean");
    assert_filter_value(&sq, Value::Bool(true));
}

#[test]
fn test_custom_config_with_multi_select() {
    let config = ParserConfig::new()
        .with_field("language", "multiSelect");
    let sq = parse_query_with_config("language:rust,python", &config)
        .and_then(|n| convert_to_structured(&n))
        .expect("should parse");
    assert_filter(&sq, "language", "includeAnyOf", "multiSelect");
    match &sq {
        StructuredQuery::Filter { value, .. } => {
            let arr = value.as_array().expect("should be array");
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Value::String("rust".to_string()));
            assert_eq!(arr[1], Value::String("python".to_string()));
        }
        _ => panic!("Expected Filter"),
    }
}

#[test]
fn test_parse_query_with_config_negation() {
    let config = ParserConfig::new()
        .with_field("rating", "number");
    let sq = parse_query_with_config("-rating:>7", &config)
        .and_then(|n| convert_to_structured(&n))
        .expect("should parse");
    assert_filter(&sq, "rating", "isLessThanOrEqualTo", "number");
    assert_filter_value(&sq, Value::from(7.0));
}

#[test]
fn test_parse_query_with_config_date_on_or_after() {
    let config = ParserConfig::new()
        .with_field("createdAt", "date");
    let sq = parse_query_with_config("createdAt:>=2024-06-01", &config)
        .and_then(|n| convert_to_structured(&n))
        .expect("should parse");
    assert_filter(&sq, "createdAt", "onOrAfter", "date");
    assert_filter_value(&sq, Value::String("2024-06-01".to_string()));
}
