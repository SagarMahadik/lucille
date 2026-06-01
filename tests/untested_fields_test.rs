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

// ── Untested boolean fields ──────────────────────────────────

#[test]
fn test_is_archived_true() {
    let sq = parse_query("isArchived:true").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "isArchived", "true", "boolean");
    assert_filter_value(&sq, Value::Bool(true));
}

#[test]
fn test_is_archived_false() {
    let sq = parse_query("isArchived:false").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "isArchived", "false", "boolean");
    assert_filter_value(&sq, Value::Bool(false));
}

#[test]
fn test_is_archived_negated() {
    let sq = parse_query("-isArchived:true").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "isArchived", "false", "boolean");
    assert_filter_value(&sq, Value::Bool(false));
}

#[test]
fn test_is_deleted_true() {
    let sq = parse_query("isDeleted:true").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "isDeleted", "true", "boolean");
    assert_filter_value(&sq, Value::Bool(true));
}

#[test]
fn test_is_deleted_false() {
    let sq = parse_query("isDeleted:false").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "isDeleted", "false", "boolean");
    assert_filter_value(&sq, Value::Bool(false));
}

#[test]
fn test_has_reminder_true() {
    let sq = parse_query("hasReminder:true").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "hasReminder", "true", "boolean");
    assert_filter_value(&sq, Value::Bool(true));
}

#[test]
fn test_has_reminder_false() {
    let sq = parse_query("hasReminder:false").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "hasReminder", "false", "boolean");
    assert_filter_value(&sq, Value::Bool(false));
}

#[test]
fn test_has_reminder_negated() {
    let sq = parse_query("-hasReminder:true").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "hasReminder", "false", "boolean");
    assert_filter_value(&sq, Value::Bool(false));
}

#[test]
fn test_with_annotation_true() {
    let sq = parse_query("withAnnotation:true").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "withAnnotation", "true", "boolean");
    assert_filter_value(&sq, Value::Bool(true));
}

#[test]
fn test_with_notes_true() {
    let sq = parse_query("withNotes:true").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "withNotes", "true", "boolean");
    assert_filter_value(&sq, Value::Bool(true));
}

#[test]
fn test_with_highlights_true() {
    let sq = parse_query("withHighlights:true").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "withHighlights", "true", "boolean");
    assert_filter_value(&sq, Value::Bool(true));
}

#[test]
fn test_with_backlinks_true() {
    let sq = parse_query("withBacklinks:true").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "withBacklinks", "true", "boolean");
    assert_filter_value(&sq, Value::Bool(true));
}

#[test]
fn test_boolean_as_0_and_1() {
    let sq1 = parse_query("isArchived:1").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq1, "isArchived", "true", "boolean");
    assert_filter_value(&sq1, Value::Bool(true));

    let sq2 = parse_query("isArchived:0").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq2, "isArchived", "false", "boolean");
    assert_filter_value(&sq2, Value::Bool(false));
}

#[test]
fn test_negated_boolean_archived_as_0() {
    let sq = parse_query("-isArchived:0").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "isArchived", "true", "boolean");
    assert_filter_value(&sq, Value::Bool(true));
}

#[test]
fn test_boolean_with_not_keyword() {
    let sq = parse_query("NOT isArchived:true").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "isArchived", "false", "boolean");
    assert_filter_value(&sq, Value::Bool(false));
}

// ── Untested date fields ─────────────────────────────────────

#[test]
fn test_log_date_after() {
    let sq = parse_query("logDate:>2024-06-01").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "logDate", "after", "date");
    assert_filter_value(&sq, Value::String("2024-06-01".to_string()));
}

#[test]
fn test_log_due_date_before() {
    let sq = parse_query("logDueDate:<2024-12-31").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "logDueDate", "before", "date");
    assert_filter_value(&sq, Value::String("2024-12-31".to_string()));
}

#[test]
fn test_due_date_exact() {
    let sq = parse_query("dueDate:2024-10-15").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "dueDate", "is", "date");
    assert_filter_value(&sq, Value::String("2024-10-15".to_string()));
}

#[test]
fn test_due_date_on_or_before() {
    let sq = parse_query("dueDate:<=2024-11-01").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "dueDate", "onOrBefore", "date");
    assert_filter_value(&sq, Value::String("2024-11-01".to_string()));
}

#[test]
fn test_reminder_time_after() {
    let sq = parse_query("reminder_time:>2024-08-01").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "reminder_time", "after", "date");
    assert_filter_value(&sq, Value::String("2024-08-01".to_string()));
}

#[test]
fn test_reminder_time_negated_after() {
    let sq = parse_query("-reminder_time:>2024-08-01").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "reminder_time", "onOrBefore", "date");
    assert_filter_value(&sq, Value::String("2024-08-01".to_string()));
}

#[test]
fn test_log_date_negated_before() {
    let sq = parse_query("-logDate:<2024-01-01").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "logDate", "onOrAfter", "date");
    assert_filter_value(&sq, Value::String("2024-01-01".to_string()));
}

#[test]
fn test_date_fields_in_boolean_expression() {
    let sq = parse_query("dueDate:>2024-01-01 AND logDate:<2024-12-31")
        .and_then(|n| convert_to_structured(&n)).unwrap();
    assert_group(&sq, "and", 2);
}

// ── Untested text fields ─────────────────────────────────────

#[test]
fn test_bookmark_id_filter() {
    let sq = parse_query("bookmarkId:bm_12345").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "bookmarkId", "is", "text");
    assert_filter_value(&sq, Value::String("bm_12345".to_string()));
}

#[test]
fn test_bookmark_id_negated() {
    let sq = parse_query("-bookmarkId:bm_12345").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "bookmarkId", "isNot", "text");
    assert_filter_value(&sq, Value::String("bm_12345".to_string()));
}

#[test]
fn test_parent_filter() {
    let sq = parse_query("parent:col_789").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "parent", "is", "text");
    assert_filter_value(&sq, Value::String("col_789".to_string()));
}

#[test]
fn test_parent_filter_with_wildcard() {
    let sq = parse_query("parent:col_*").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "parent", "startsWith", "text");
    assert_filter_value(&sq, Value::String("col_".to_string()));
}

#[test]
fn test_parent_negated() {
    let sq = parse_query("-parent:col_789").and_then(|n| convert_to_structured(&n)).unwrap();
    assert_filter(&sq, "parent", "isNot", "text");
    assert_filter_value(&sq, Value::String("col_789".to_string()));
}

// ── Helper for group assertion ───────────────────────────────

fn assert_group(sq: &StructuredQuery, expected_operator: &str, expected_count: usize) {
    match sq {
        StructuredQuery::Group { operator, filters } => {
            assert_eq!(operator, expected_operator, "operator mismatch");
            assert_eq!(filters.len(), expected_count, "filter count mismatch");
        }
        other => panic!("Expected Group, got {:?}", other),
    }
}
