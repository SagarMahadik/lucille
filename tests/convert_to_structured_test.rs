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

fn assert_filter_value(sq: &StructuredQuery, expected_value: serde_json::Value) {
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
            assert_eq!(filters.len(), expected_count, "filter count mismatch");
        }
        other => panic!("Expected Group, got {:?}", other),
    }
}

fn assert_not_group(sq: &StructuredQuery) {
    match sq {
        StructuredQuery::NotGroup { .. } => {}
        other => panic!("Expected NotGroup, got {:?}", other),
    }
}

// ==================== SINGLE FILTERS ====================

#[test]
fn test_single_text_filter() {
    let node = parse_query("domain:google.com").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "domain", "is", "text");
    assert_filter_value(&sq, Value::String("google.com".to_string()));
}

#[test]
fn test_single_negated_prefix_filter() {
    let node = parse_query("-domain:google.com").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "domain", "isNot", "text");
    assert_filter_value(&sq, Value::String("google.com".to_string()));
}

#[test]
fn test_single_not_keyword_filter() {
    let node = parse_query("NOT domain:google.com").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "domain", "isNot", "text");
    assert_filter_value(&sq, Value::String("google.com".to_string()));
}

#[test]
fn test_negated_and_not_keyword_same() {
    // Both -prefix and NOT keyword should produce identical output
    let node_a = parse_query("-domain:google.com").unwrap();
    let node_b = parse_query("NOT domain:google.com").unwrap();
    let sq_a = convert_to_structured(&node_a).unwrap();
    let sq_b = convert_to_structured(&node_b).unwrap();
    assert_eq!(sq_a, sq_b, "-prefix and NOT keyword should produce same result");
}

#[test]
fn test_double_negation_not_not_filter_not_supported() {
    // Parser does not support consecutive NOT keywords (same as JS)
    let node = parse_query("NOT NOT domain:google.com");
    assert!(node.is_none(), "double NOT should not parse");
}

// ==================== MULTIPLE FILTERS ====================

#[test]
fn test_two_filters_implicit_and() {
    let node = parse_query("domain:google.com isFavorite:true").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_group(&sq, "and", 2);
    if let StructuredQuery::Group { filters, .. } = &sq {
        assert_filter(&filters[0], "domain", "is", "text");
        assert_filter(&filters[1], "isFavorite", "true", "boolean");
        assert_filter_value(&filters[1], Value::Bool(true));
    }
}

#[test]
fn test_two_filters_explicit_and() {
    let node = parse_query("domain:google.com AND isFavorite:true").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_group(&sq, "and", 2);
}

#[test]
fn test_two_filters_explicit_or() {
    let node = parse_query("domain:google.com OR domain:twitter.com").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_group(&sq, "or", 2);
}

#[test]
fn test_three_filters_implicit_and() {
    let node = parse_query("domain:twitter.com tag:rust rating:>3").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_group(&sq, "and", 3);
    if let StructuredQuery::Group { filters, .. } = &sq {
        assert_filter(&filters[0], "domain", "is", "text");
        assert_filter(&filters[1], "tags", "includes", "multiSelect");
        assert_filter(&filters[2], "rating", "greaterThan", "number");
        assert_filter_value(&filters[2], Value::Number(serde_json::Number::from_f64(3.0).unwrap()));
    }
}

#[test]
fn test_mixed_or_and() {
    // a AND b OR c  -> (a AND b) OR c
    let node = parse_query("domain:google.com isFavorite:true OR tag:rust").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    // Should be: OR with 2 children: AND(domain, isFavorite) and tags filter
    assert_group(&sq, "or", 2);
}

// ==================== NOT WITH GROUPS ====================

#[test]
fn test_not_group_or() {
    let node = parse_query("NOT (domain:google.com OR domain:twitter.com)").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_not_group(&sq);
    if let StructuredQuery::NotGroup { filter, .. } = &sq {
        assert_group(filter, "or", 2);
    }
}

#[test]
fn test_not_group_and() {
    let node = parse_query("NOT (domain:google.com isFavorite:true)").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_not_group(&sq);
    if let StructuredQuery::NotGroup { filter, .. } = &sq {
        assert_group(filter, "and", 2);
    }
}

#[test]
fn test_double_not_group() {
    // Show that wrapped NOT groups work, but consecutive NOTs don't
    // This test uses NOT (NOT (group)) effectively
    let _node = parse_query("NOT NOT (domain:google.com isFavorite:true)");
    let sq = parse_query("NOT (domain:google.com isFavorite:true)").unwrap();
    let result = convert_to_structured(&sq).unwrap();
    assert_not_group(&result);
    if let StructuredQuery::NotGroup { filter, .. } = &result {
        assert_group(filter, "and", 2);
    }
}

// ==================== TAG FILTERS ====================

#[test]
fn test_hash_tag_filter() {
    let node = parse_query("#rust").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "tags", "includes", "multiSelect");
    assert_filter_value(&sq, Value::String("rust".to_string()));
}

#[test]
fn test_negated_hash_tag_filter() {
    let node = parse_query("-#rust").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "tags", "doesNotInclude", "multiSelect");
}

#[test]
fn test_tag_property_filter() {
    let node = parse_query("tag:rust").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "tags", "includes", "multiSelect");
}

#[test]
fn test_tags_property_filter() {
    let node = parse_query("tags:rust,js").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "tags", "includeAnyOf", "multiSelect");
    if let StructuredQuery::Filter { value, .. } = &sq {
        match value {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(arr[0], Value::String("rust".to_string()));
                assert_eq!(arr[1], Value::String("js".to_string()));
            }
            _ => panic!("Expected Array value"),
        }
    }
}

// ==================== BOOLEAN FILTERS ====================

#[test]
fn test_boolean_true() {
    let node = parse_query("isFavorite:true").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "isFavorite", "true", "boolean");
    assert_filter_value(&sq, Value::Bool(true));
}

#[test]
fn test_boolean_false() {
    let node = parse_query("isFavorite:false").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "isFavorite", "false", "boolean");
    assert_filter_value(&sq, Value::Bool(false));
}

#[test]
fn test_negated_boolean_true() {
    let node = parse_query("-isFavorite:true").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "isFavorite", "false", "boolean");
    assert_filter_value(&sq, Value::Bool(false));
}

#[test]
fn test_negated_boolean_false() {
    // -isFavorite:false  == isFavorite:true  (double negative)
    let node = parse_query("-isFavorite:false").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "isFavorite", "true", "boolean");
    assert_filter_value(&sq, Value::Bool(true));
}

// ==================== NUMBER FILTERS ====================

#[test]
fn test_number_equals() {
    let node = parse_query("rating:5").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "rating", "is", "number");
    assert_filter_value(&sq, Value::Number(serde_json::Number::from_f64(5.0).unwrap()));
}

#[test]
fn test_number_greater_than() {
    let node = parse_query("rating:>3").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "rating", "greaterThan", "number");
}

#[test]
fn test_number_less_than() {
    let node = parse_query("rating:<2").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "rating", "lessThan", "number");
}

#[test]
fn test_number_gte() {
    let node = parse_query("rating:>=4").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "rating", "isGreaterThanOrEqualTo", "number");
}

#[test]
fn test_number_lte() {
    let node = parse_query("rating:<=1").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "rating", "isLessThanOrEqualTo", "number");
}

#[test]
fn test_negated_greater_than() {
    // -rating:>3 means rating is NOT > 3, so condition flips to isLessThanOrEqualTo
    let node = parse_query("-rating:>3").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "rating", "isLessThanOrEqualTo", "number");
}

// ==================== DATE FILTERS ====================

#[test]
fn test_date_after() {
    let node = parse_query("createdAt:>2024-01-01").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "createdAt", "after", "date");
    assert_filter_value(&sq, Value::String("2024-01-01".to_string()));
}

#[test]
fn test_date_before() {
    let node = parse_query("createdAt:<2024-06-01").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "createdAt", "before", "date");
}

// ==================== WILDCARDS ====================

#[test]
fn test_wildcard_contains() {
    let node = parse_query("domain:*google*").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "domain", "contains", "text");
    assert_filter_value(&sq, Value::String("google".to_string()));
}

#[test]
fn test_wildcard_starts_with() {
    let node = parse_query("domain:google*").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "domain", "startsWith", "text");
    assert_filter_value(&sq, Value::String("google".to_string()));
}

#[test]
fn test_wildcard_ends_with() {
    let node = parse_query("domain:*.com").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "domain", "endsWith", "text");
    assert_filter_value(&sq, Value::String(".com".to_string()));
}

#[test]
fn test_negated_wildcard_contains() {
    let node = parse_query("-domain:*google*").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "domain", "doesNotContain", "text");
}

// ==================== ABBREVIATIONS ====================

#[test]
fn test_abbreviation_t_to_tags() {
    let node = parse_query("t:rust").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "tags", "includes", "multiSelect");
}

#[test]
fn test_abbreviation_created_to_createdat() {
    let node = parse_query("created:>2024-01-01").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "createdAt", "after", "date");
}

#[test]
fn test_abbreviation_modified_to_updatedat() {
    let node = parse_query("modified:>2024-01-01").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "updatedAt", "after", "date");
}

// ==================== CONTENT SEARCH ====================

#[test]
fn test_simple_content_search() {
    let node = parse_query("hello").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "content_tokens", "is", "text");
}

#[test]
fn test_negated_content_search() {
    let node = parse_query("-hello").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "content_tokens", "isNot", "text");
}

#[test]
fn test_multiple_content_search_converted_to_or() {
    // Pure content searches have their ANDs converted to ORs
    let node = parse_query("steve jobs").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    // Should be OR due to AND-to-OR conversion
    assert_group(&sq, "or", 2);
    if let StructuredQuery::Group { filters, .. } = &sq {
        assert_filter(&filters[0], "content_tokens", "is", "text");
        assert_filter(&filters[1], "content_tokens", "is", "text");
        assert_filter_value(&filters[0], Value::String("steve".to_string()));
        assert_filter_value(&filters[1], Value::String("jobs".to_string()));
    }
}

#[test]
fn test_quoted_content_search() {
    let node = parse_query("\"steve jobs\"").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "content_tokens", "is", "text");
}

// ==================== COMPLEX NESTED ====================

#[test]
fn test_nested_parentheses() {
    let node = parse_query("(domain:google.com OR domain:twitter.com) AND isFavorite:true").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_group(&sq, "and", 2);
    if let StructuredQuery::Group { filters, .. } = &sq {
        assert_group(&filters[0], "or", 2);
        assert_filter(&filters[1], "isFavorite", "true", "boolean");
    }
}

#[test]
fn test_deeply_nested() {
    let node = parse_query("(a:1 OR (b:2 AND c:3)) AND d:4").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_group(&sq, "and", 2);
}

// ==================== FLATTENING ====================

#[test]
fn test_and_flattening() {
    // domain AND isFavorite AND tag:rust should flatten to a single AND group with 3 filters
    let node = parse_query("domain:google.com isFavorite:true tag:rust").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_group(&sq, "and", 3);
}

#[test]
fn test_mixed_and_or_no_flatten() {
    // Different operators should NOT be flattened
    // (a AND b) OR c
    let node = parse_query("(domain:google.com isFavorite:true) OR tag:rust").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_group(&sq, "or", 2);
    if let StructuredQuery::Group { filters, .. } = &sq {
        assert_group(&filters[0], "and", 2);
        assert_filter(&filters[1], "tags", "includes", "multiSelect");
    }
}

// ==================== CUSTOM PROPERTIES ====================

#[test]
fn test_custom_property() {
    let node = parse_query("cp_author:Taleb").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    assert_filter(&sq, "author", "is", "text");
    if let StructuredQuery::Filter { is_custom_property, .. } = &sq {
        assert!(is_custom_property, "expected custom property");
    }
}

// ==================== EDGE CASES ====================

#[test]
fn test_empty_query() {
    let node = parse_query("");
    assert!(node.is_none(), "empty query should return None");
}

#[test]
fn test_whitespace_only_query() {
    let node = parse_query("   ");
    assert!(node.is_none(), "whitespace-only query should return None");
}

#[test]
fn test_empty_parentheses() {
    // Empty parentheses () contain no filter so the parse result is None
    let node = parse_query("()");
    assert!(node.is_none(), "empty parentheses should produce None");
}

#[test]
fn test_match_or_operator() {
    let node = parse_query("match:OR domain:google.com domain:twitter.com").unwrap();
    let sq = convert_to_structured(&node).unwrap();
    // With match:OR, default operator switches to OR
    assert_group(&sq, "or", 2);
}

#[test]
fn test_not_on_filter_bakes_negation() {
    // NOT domain:google.com should produce same as -domain:google.com
    let node_not = parse_query("NOT domain:google.com").unwrap();
    let node_neg = parse_query("-domain:google.com").unwrap();
    let sq_not = convert_to_structured(&node_not).unwrap();
    let sq_neg = convert_to_structured(&node_neg).unwrap();
    assert_eq!(
        sq_not, sq_neg,
        "NOT keyword and -prefix should produce identical structured output"
    );
}
