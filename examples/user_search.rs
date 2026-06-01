/// Example: Building a typed filter from user input (no LLM).
///
/// Parse a user-typed search query and use the typed AST to
/// construct SQL WHERE clauses safely.
use lucille::{parse_query, convert_to_structured, StructuredQuery};

fn main() {
    let user_query = "domain:example.com isArchived:false createdAt:>2024-06-01";

    let ast = parse_query(user_query).expect("user query should parse");
    let structured = convert_to_structured(&ast).expect("should convert");

    println!("User typed: {user_query}");
    println!("Parsed AST:\n{}\n", serde_json::to_string_pretty(&structured).unwrap());

    // Demo: walk the AST and classify filters
    if let StructuredQuery::Group { operator, filters } = &structured {
        println!("Query has {operator} group with {} filters:", filters.len());
        for f in filters {
            if let StructuredQuery::Filter { field, condition, field_type, .. } = f {
                println!("  {field} ({field_type}) must {condition}");
            }
        }
    }
}
