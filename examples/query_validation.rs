/// Example: Validating LLM output before query execution.
///
/// Demonstrates catching malformed or unsafe LLM-generated queries
/// before they reach your database.
use lucille::{parse_query, convert_to_structured};

fn main() {
    // Safe — LLM output is valid Lucille syntax
    let safe = parse_query("type:book rating:>4");
    match safe.and_then(|n| convert_to_structured(&n)) {
        Some(_) => println!("✅ Safe query accepted"),
        None => println!("❌ Safe query unexpectedly rejected"),
    }

    // Malformed — unclosed paren, still parses but edge case
    let malformed = parse_query("type:book (rating:>4");
    match malformed.and_then(|n| convert_to_structured(&n)) {
        Some(ast) => println!("⚠️  Malformed query produced AST (lenient): {}", serde_json::to_string(&ast).unwrap()),
        None => println!("✅ Malformed query correctly rejected"),
    }

    // Empty / no-op
    let empty = parse_query("");
    assert!(empty.is_none(), "empty query should yield None");

    println!("→ Lucille acts as a safety layer between LLM output and query execution");
}
