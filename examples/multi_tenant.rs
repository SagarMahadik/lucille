/// Example: Multi-tenant search with per-tenant schemas.
///
/// Each tenant defines their own searchable fields and abbreviations.
/// The LLM generates tenant-scoped queries; Lucille validates per-tenant schema.
use lucille::{ParserConfig, parse_query_with_config, convert_to_structured};

fn main() {
    // Tenant A: issue tracker
    let tenant_a = ParserConfig::new()
        .with_field("status", "multiSelect")
        .with_field("assignee", "text")
        .with_field("priority", "number")
        .with_field("createdAt", "date")
        .with_abbreviation("p", "priority")
        .with_abbreviation("cr", "createdAt");

    let query_a = "status:open,active priority:>5 assignee:me";
    let ast = parse_query_with_config(query_a, &tenant_a).expect("tenant A query");
    println!("Tenant A:\n{}\n", serde_json::to_string_pretty(&convert_to_structured(&ast)).unwrap());

    // Tenant B: e-commerce catalog
    let tenant_b = ParserConfig::new()
        .with_field("category", "multiSelect")
        .with_field("price", "number")
        .with_field("inStock", "boolean")
        .with_field("brand", "text")
        .with_abbreviation("cat", "category");

    let query_b = "category:electronics price:<500 inStock:true";
    let ast = parse_query_with_config(query_b, &tenant_b).expect("tenant B query");
    println!("Tenant B:\n{}\n", serde_json::to_string_pretty(&convert_to_structured(&ast)).unwrap());
}
