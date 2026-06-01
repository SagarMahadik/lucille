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
            assert_eq!(field, expected_field, "field mismatch. Got '{}', expected '{}'", field, expected_field);
            assert_eq!(condition, expected_condition, "condition mismatch. Got '{}', expected '{}'", condition, expected_condition);
            assert_eq!(field_type, expected_field_type, "field_type mismatch. Got '{}', expected '{}'", field_type, expected_field_type);
        }
        other => panic!("Expected Filter, got {:?}", other),
    }
}

fn assert_filter_value(sq: &StructuredQuery, expected: impl Into<Value>) {
    let expected_value = expected.into();
    match sq {
        StructuredQuery::Filter { value, .. } => {
            assert_eq!(value, &expected_value, "value mismatch");
        }
        other => panic!("Expected Filter, got {:?}", other),
    }
}

fn assert_group(sq: &StructuredQuery, expected_operator: &str, expected_count: usize) {
    match sq {
        StructuredQuery::Group {
            operator,
            filters,
        } => {
            assert_eq!(operator, expected_operator, "operator mismatch");
            assert_eq!(filters.len(), expected_count, "filter count mismatch. Expected {expected_count}, got {}", filters.len());
        }
        other => panic!("Expected Group, got {:?}", other),
    }
}

// ==================== COMPLEX BOOLEAN EXPRESSIONS ====================

#[test]
fn test_nested_or_inside_and() {
    let node = parse_query("(domain:google.com OR domain:twitter.com) AND isFavorite:true").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_group(&sq, "and", 2);
    if let StructuredQuery::Group { filters, .. } = &sq {
        assert_group(&filters[0], "or", 2);
        assert_filter(&filters[1], "isFavorite", "true", "boolean");
    }
}

#[test]
fn test_nested_and_inside_or() {
    let node = parse_query("(domain:google.com isFavorite:true) OR tag:rust").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_group(&sq, "or", 2);
    if let StructuredQuery::Group { filters, .. } = &sq {
        assert_group(&filters[0], "and", 2);
        assert_filter(&filters[1], "tags", "includes", "multiSelect");
    }
}

#[test]
fn test_complex_nested_groups() {
    let node = parse_query("(a:1 OR b:2) AND (c:3 OR d:4)").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_group(&sq, "and", 2);
    if let StructuredQuery::Group { filters, .. } = &sq {
        assert_group(&filters[0], "or", 2);
        assert_group(&filters[1], "or", 2);
    }
}

#[test]
fn test_triple_nested_group() {
    let node = parse_query("((a:1 AND b:2) OR c:3) AND d:4").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_group(&sq, "and", 2);
    if let StructuredQuery::Group { filters, .. } = &sq {
        assert_group(&filters[0], "or", 2);
        if let StructuredQuery::Group { filters: or_filters, .. } = &filters[0] {
            assert_group(&or_filters[0], "and", 2);
            assert_filter(&or_filters[1], "c", "is", "text");
        }
        assert_filter(&filters[1], "d", "is", "text");
    }
}

// ==================== MULTIPLE NEGATIONS ====================

#[test]
fn test_two_negated_filters() {
    let node = parse_query("-domain:google.com -isFavorite:true").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_group(&sq, "and", 2);
    if let StructuredQuery::Group { filters, .. } = &sq {
        assert_filter(&filters[0], "domain", "isNot", "text");
        assert_filter(&filters[1], "isFavorite", "false", "boolean");
    }
}

#[test]
fn test_negated_filter_and_positive_filter() {
    let node = parse_query("-domain:google.com isFavorite:true").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_group(&sq, "and", 2);
    if let StructuredQuery::Group { filters, .. } = &sq {
        assert_filter(&filters[0], "domain", "isNot", "text");
        assert_filter(&filters[1], "isFavorite", "true", "boolean");
    }
}

// ==================== QUOTED STRINGS ====================

#[test]
fn test_quoted_tag_value_keeps_quotes() {
    // Rust parser keeps surrounding quotes in the value (unlike JS which strips them)
    let node = parse_query("tag:\"machine learning\"").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "tags", "includes", "multiSelect");
    assert_filter_value(&sq, Value::String("\"machine learning\"".to_string()));
}

#[test]
fn test_quoted_text_value() {
    let node = parse_query("title:\"hello world\"").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "title_tokens", "is", "text");
    // Quoted string is run through hybrid_tokenize, so "hello world" becomes "hello world"
    assert_filter_value(&sq, Value::String("hello world".to_string()));
}

#[test]
fn test_negative_quoted_content() {
    let node = parse_query("-\"hello world\"").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "content_tokens", "isNot", "text");
}

// ==================== COMMA-SEPARATED VALUES ====================

#[test]
fn test_comma_separated_text_values() {
    let node = parse_query("type:article,book").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "type", "isAnyOf", "text");
    if let StructuredQuery::Filter { value, .. } = &sq {
        match value {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(arr[0], "article");
                assert_eq!(arr[1], "book");
            }
            _ => panic!("Expected Array"),
        }
    }
}

#[test]
fn test_comma_separated_tags() {
    let node = parse_query("tags:rust,typescript,go").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "tags", "includeAnyOf", "multiSelect");
    if let StructuredQuery::Filter { value, .. } = &sq {
        match value {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
            }
            _ => panic!("Expected Array"),
        }
    }
}

#[test]
fn test_single_tag_value() {
    // Single tag should use "includes" not "includeAnyOf"
    let node = parse_query("tags:rust").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "tags", "includes", "multiSelect");
    assert_filter_value(&sq, Value::String("rust".to_string()));
}

// ==================== FIELD TYPE MAPPINGS (FTS fields) ====================

#[test]
fn test_title_field_maps_to_title_tokens() {
    let node = parse_query("title:hello").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "title_tokens", "is", "text");
}

#[test]
fn test_description_field_maps_to_desc_tokens() {
    let node = parse_query("description:test").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "desc_tokens", "is", "text");
}

#[test]
fn test_url_field_maps_to_url_tokens() {
    let node = parse_query("url:example.com").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "url_tokens", "is", "text");
}

#[test]
fn test_note_field_maps_to_note_tokens() {
    let node = parse_query("note:reminder").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "note_tokens", "is", "text");
}

#[test]
fn test_content_text_maps_to_content_tokens() {
    let node = parse_query("text:search").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "text_tokens", "is", "text");
}

// ==================== BOOLEAN NUMERIC VALUES ====================

#[test]
fn test_boolean_as_0() {
    let node = parse_query("isFavorite:0").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "isFavorite", "false", "boolean");
    assert_filter_value(&sq, Value::Bool(false));
}

#[test]
fn test_boolean_as_1() {
    let node = parse_query("isFavorite:1").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "isFavorite", "true", "boolean");
    assert_filter_value(&sq, Value::Bool(true));
}

#[test]
fn test_negated_boolean_as_0() {
    // -isFavorite:0  →  isFavorite:true (double negative)
    let node = parse_query("-isFavorite:0").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "isFavorite", "true", "boolean");
}

// ==================== NUMBER EDGE CASES ====================

#[test]
fn test_number_negative_value() {
    let node = parse_query("rating:-1").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "rating", "is", "number");
    assert_filter_value(&sq, Value::Number(serde_json::Number::from_f64(-1.0).unwrap()));
}

#[test]
fn test_number_decimal() {
    let node = parse_query("rating:3.5").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "rating", "is", "number");
    assert_filter_value(&sq, Value::Number(serde_json::Number::from_f64(3.5).unwrap()));
}

#[test]
fn test_number_gte_negative() {
    let node = parse_query("rating:>=-5").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "rating", "isGreaterThanOrEqualTo", "number");
}

#[test]
fn test_number_zero() {
    let node = parse_query("rating:0").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "rating", "is", "number");
    assert_filter_value(&sq, Value::Number(serde_json::Number::from_f64(0.0).unwrap()));
}

// ==================== DATE EDGE CASES ====================

#[test]
fn test_date_on_or_after() {
    let node = parse_query("createdAt:>=2024-06-15").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "createdAt", "onOrAfter", "date");
}

#[test]
fn test_date_on_or_before() {
    let node = parse_query("createdAt:<=2024-12-31").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "createdAt", "onOrBefore", "date");
}

#[test]
fn test_date_exact_match() {
    // Rust parser defaults to "is" for date without operator (not "in" like JS)
    let node = parse_query("createdAt:2024-01-01").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "createdAt", "is", "date");
}

#[test]
fn test_negated_date_after() {
    let node = parse_query("-createdAt:>2024-01-01").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "createdAt", "onOrBefore", "date");
}

// ==================== MULTI-SELECT EDGE CASES ====================

#[test]
fn test_ai_tags_field() {
    let node = parse_query("aiTags:tech").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "aiTags", "includes", "multiSelect");
}

#[test]
fn test_negated_multi_select() {
    let node = parse_query("-tags:rust").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "tags", "doesNotInclude", "multiSelect");
}

#[test]
fn test_exclude_any_tag() {
    let node = parse_query("-tags:rust,js").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "tags", "excludeAnyOf", "multiSelect");
}

#[test]
fn test_nonexistent_field_defaults_to_text() {
    let node = parse_query("someUnknownField:hello").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "someUnknownField", "is", "text");
}

// ==================== CUSTOM PROPERTIES ====================

#[test]
fn test_custom_property_is_custom_flag() {
    let node = parse_query("cp_isbn:9780143039433").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    match &sq {
        StructuredQuery::Filter { is_custom_property, .. } => {
            assert!(is_custom_property, "expected custom property flag true");
        }
        _ => panic!("Expected Filter"),
    }
}

#[test]
fn test_regular_property_not_custom() {
    let node = parse_query("domain:google.com").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    match &sq {
        StructuredQuery::Filter { is_custom_property, .. } => {
            assert!(!is_custom_property, "expected custom property flag false");
        }
        _ => panic!("Expected Filter"),
    }
}

// ==================== WHITESPACE HANDLING ====================

#[test]
fn test_leading_whitespace() {
    let node = parse_query("   domain:google.com").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "domain", "is", "text");
}

#[test]
fn test_trailing_whitespace() {
    let node = parse_query("domain:google.com   ").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "domain", "is", "text");
}

#[test]
fn test_extra_spaces_between_filters() {
    let node = parse_query("domain:google.com     isFavorite:true").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_group(&sq, "and", 2);
}

// ==================== SPECIAL CHARACTERS ====================

#[test]
fn test_underscore_in_field_name() {
    let node = parse_query("file_format:pdf").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "file_format", "is", "text");
}

#[test]
fn test_domain_with_subdomain() {
    let node = parse_query("domain:sub.example.com").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "domain", "is", "text");
    assert_filter_value(&sq, Value::String("sub.example.com".to_string()));
}

#[test]
fn test_domain_with_port_is_two_filters() {
    // The colon in ":8080" is a property delimiter, so this splits into
    // domain:localhost AND content:8080
    let node = parse_query("domain:localhost:8080").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_group(&sq, "and", 2);
    if let StructuredQuery::Group { filters, .. } = &sq {
        assert_filter(&filters[0], "domain", "is", "text");
        assert_filter_value(&filters[0], Value::String("localhost".to_string()));
        assert_filter(&filters[1], "content_tokens", "is", "text");
        assert_filter_value(&filters[1], Value::String("8080".to_string()));
    }
}

// ==================== OBJECT TYPE ====================

#[test]
fn test_object_type_filter() {
    let node = parse_query("objectType:movie").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "objectType", "is", "text");
}

// ==================== MIXED CONTENT + FIELD SEARCH ====================

#[test]
fn test_mixed_content_and_field_search() {
    let node = parse_query("hello domain:google.com").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    // Content search AND field filter — should remain AND (not converted to OR since not pure content)
    assert_group(&sq, "and", 2);
    if let StructuredQuery::Group { filters, .. } = &sq {
        assert_filter(&filters[0], "content_tokens", "is", "text");
        assert_filter(&filters[1], "domain", "is", "text");
    }
}

#[test]
fn test_three_mixed_content_different_fields() {
    let node = parse_query("api domain:google.com tag:rust").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_group(&sq, "and", 3);
}

// ==================== AND-TO-OR CONVERSION EDGE CASES ====================

#[test]
fn test_pure_content_conversion_not_with_field_mix() {
    // Multiple content terms + field filter should NOT convert ANDs to ORs
    let node = parse_query("hello world domain:google.com").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    // Because there's a non-content filter, AND-to-OR conversion should not happen
    // The AST is: (content AND content) AND domain → all AND
    assert_group(&sq, "and", 3);
}

#[test]
fn test_pure_content_conversion_happens() {
    // Only content terms — should convert to OR
    let node = parse_query("hello world foo bar").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_group(&sq, "or", 4);
}

// ==================== READ STATUS / STATUS / RATING ====================

#[test]
fn test_read_status_number() {
    let node = parse_query("readStatus:3").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "readStatus", "is", "number");
}

#[test]
fn test_read_status_greater_than() {
    let node = parse_query("readStatus:>2").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "readStatus", "greaterThan", "number");
}

#[test]
fn test_status_number() {
    let node = parse_query("status:1").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "status", "is", "number");
}

// ==================== CREATED/UPDATED/TIME ====================

#[test]
fn test_updatedat_date() {
    let node = parse_query("updatedAt:>2024-06-01").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "updatedAt", "after", "date");
}

#[test]
fn test_time_date() {
    let node = parse_query("time:>2024-01-01").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "time", "after", "date");
}

// ==================== FILE SIZE / DURATION ====================

#[test]
fn test_file_size_number() {
    let node = parse_query("fileSize:1024").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "fileSize", "is", "number");
}

#[test]
fn test_duration_number() {
    let node = parse_query("duration:120").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "duration", "is", "number");
}

// ==================== PINNED COLLECTIONS ====================

#[test]
fn test_pinned_collections_is_text_default() {
    // pinnedCollections is not in Rust's FIELD_TYPES, defaults to text
    let node = parse_query("pinnedCollections:bookmarks").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "pinnedCollections", "is", "text");
}

// ==================== HIGHLIGHT LENGTH AND OTHER TEXT FIELDS ====================

#[test]
fn test_highlight_length_text() {
    let node = parse_query("highlightLength:>50").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    let _ = sq; // just check it parses
}

// ==================== SERIALIZATION ROUND-TRIP ====================

#[test]
fn test_serialize_deserialize_filter() {
    let sq = StructuredQuery::Filter {
        field: "domain".into(),
        condition: "is".into(),
        value: Value::String("google.com".into()),
        field_type: "text".into(),
        is_custom_property: false,
    };
    let json = serde_json::to_string(&sq).unwrap();
    let back: StructuredQuery = serde_json::from_str(&json).unwrap();
    assert_eq!(sq, back);
}

#[test]
fn test_serialize_deserialize_group() {
    let sq = StructuredQuery::Group {
        operator: "and".into(),
        filters: vec![
            StructuredQuery::Filter {
                field: "a".into(),
                condition: "is".into(),
                value: Value::String("1".into()),
                field_type: "text".into(),
                is_custom_property: false,
            },
            StructuredQuery::Filter {
                field: "b".into(),
                condition: "is".into(),
                value: Value::String("2".into()),
                field_type: "text".into(),
                is_custom_property: false,
            },
        ],
    };
    let json = serde_json::to_string_pretty(&sq).unwrap();
    let back: StructuredQuery = serde_json::from_str(&json).unwrap();
    assert_eq!(sq, back);
}

#[test]
fn test_serialize_deserialize_not_group() {
    let sq = StructuredQuery::NotGroup {
        operator: "not".into(),
        filter: Box::new(StructuredQuery::Filter {
            field: "domain".into(),
            condition: "is".into(),
            value: Value::String("google.com".into()),
            field_type: "text".into(),
            is_custom_property: false,
        }),
    };
    let json = serde_json::to_string(&sq).unwrap();
    let back: StructuredQuery = serde_json::from_str(&json).unwrap();
    assert_eq!(sq, back);
}

#[test]
fn test_structured_json_output_format() {
    // Verify the JSON output matches the expected shape
    let node = parse_query("domain:google.com isFavorite:true").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    let json = serde_json::to_value(&sq).unwrap();

    // Should be an object with operator and filters
    assert!(json.is_object());
    assert_eq!(json["operator"], "and");
    assert!(json["filters"].is_array());
    assert_eq!(json["filters"].as_array().unwrap().len(), 2);

    // First filter
    let f0 = &json["filters"][0];
    assert_eq!(f0["field"], "domain");
    assert_eq!(f0["condition"], "is");
    assert_eq!(f0["value"], "google.com");
    assert_eq!(f0["fieldType"], "text");
    assert_eq!(f0["isCustomProperty"], false);

    // Second filter — Rust uses string "true"/"false" for boolean condition (unlike JS which uses bool)
    let f1 = &json["filters"][1];
    assert_eq!(f1["field"], "isFavorite");
    assert_eq!(f1["condition"], "true");
    assert_eq!(f1["value"], true);
    assert_eq!(f1["fieldType"], "boolean");
    assert_eq!(f1["isCustomProperty"], false);
}

#[test]
fn test_not_group_json_output_format() {
    let node = parse_query("NOT (domain:google.com OR domain:twitter.com)").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    let json = serde_json::to_value(&sq).unwrap();

    assert!(json.is_object());
    assert_eq!(json["operator"], "not");
    assert!(json.get("filter").is_some());
    assert_eq!(json["filter"]["operator"], "or");
    assert_eq!(json["filter"]["filters"].as_array().unwrap().len(), 2);
}

// ==================== EMPTY / NULL EDGE CASES ====================

#[test]
fn test_single_char_query() {
    // Single char should parse as content search
    let node = parse_query("x").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "content_tokens", "is", "text");
}

#[test]
fn test_query_with_only_not_operator_parses_as_content() {
    // "NOT" at end of input becomes a content search, not NotOperator
    let node = parse_query("NOT");
    assert!(node.is_some(), "standalone NOT parses as content search");
    let sq = convert_to_structured(&node.unwrap()).unwrap();
    assert_filter(&sq, "content_tokens", "is", "text");
    // The value is "not" (lowercased by hybrid_tokenize)
}

#[test]
fn test_not_at_end_of_query() {
    let node = parse_query("domain:google.com NOT");
    // NOT at end with nothing after it should fall through
    assert!(node.is_some());
}

// ==================== MATCH:OR COMBINATIONS ====================

#[test]
fn test_match_or_with_not() {
    let node = parse_query("match:OR -domain:google.com domain:twitter.com").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    // With match:OR, two filters should be OR'd
    assert_group(&sq, "or", 2);
    if let StructuredQuery::Group { filters, .. } = &sq {
        assert_filter(&filters[0], "domain", "isNot", "text");
        assert_filter(&filters[1], "domain", "is", "text");
    }
}

// ==================== PARENTHESES WITH NEGATION ====================

#[test]
fn test_negation_before_paren_is_content_search() {
    // The - before ( is not treated as group negation — it becomes a
    // negative content search for the literal "-(" characters
    let node = parse_query("-(domain:google.com isFavorite:true)").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    // Parses as content filter(s) — should not panic
    let _ = sq; // just verify it doesn't crash
}

#[test]
fn test_single_filter_in_parens() {
    let node = parse_query("(domain:google.com)").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    // Single filter in parens should unwrap to just the filter
    assert_filter(&sq, "domain", "is", "text");
}

// ==================== FIELD NAME CASE INSENSITIVITY ====================

#[test]
fn test_uppercase_field_name_preserves_case() {
    // Rust parser preserves original field name case (like JS)
    let node = parse_query("DOMAIN:google.com").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "DOMAIN", "is", "text");
}

#[test]
fn test_mixed_case_field_name_preserves_case() {
    // Field name case is preserved; field type lookup is case-sensitive
    let node = parse_query("IsFavorite:true").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "IsFavorite", "is", "text");
}

// ==================== VALUE WITH HYPHENS ====================

#[test]
fn test_value_with_hyphens() {
    let node = parse_query("type:well-known").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "type", "is", "text");
    assert_filter_value(&sq, Value::String("well-known".to_string()));
}

// ==================== MULTIPLE WILDCARDS ====================

#[test]
fn test_wildcard_only_star_becomes_contains_empty() {
    let node = parse_query("domain:*").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "domain", "contains", "text");
    assert_filter_value(&sq, Value::String("".to_string()));
}
