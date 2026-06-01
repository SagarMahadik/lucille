/// Example: LLM-safe RAG filter pipeline.
///
/// Instead of letting an LLM generate raw SQL, parse its output through Lucille
/// to get a validated, typed AST before executing against any backend.
use lucille::{parse_query, convert_to_structured, StructuredQuery};

fn main() {
    // Simulated: LLM output for "show my favorite Rust articles from the last two years"
    let llm_query = "type:article tag:rust isFavorite:true createdAt:>2024-01-01";

    let ast = parse_query(llm_query).expect("LLM output should be parseable");
    let structured = convert_to_structured(&ast).expect("should convert");

    println!("LLM generated query: {llm_query}");
    println!("Lucille AST:\n{}\n", serde_json::to_string_pretty(&structured).unwrap());

    // Now safely translate to any backend (SQL, Elastic, in-memory, etc.)
    match &structured {
        StructuredQuery::Group { operator, filters } => {
            println!("→ Safe to translate: {} filters joined by {}", filters.len(), operator);
        }
        _ => println!("→ Single filter, safe to translate"),
    }
}
