use lucille::{ParserConfig, parse_query, parse_query_with_config, convert_to_structured};

fn main() {
    println!("=== Default config ===");
    let queries = [
        "domain:google.com isFavorite:true",
        "t:rust rating:>3",
    ];
    for query in &queries {
        println!("Query: {query}");
        let node = parse_query(query).unwrap();
        let structured = convert_to_structured(&node);
        println!("{}\n", serde_json::to_string_pretty(&structured).unwrap());
    }

    println!("=== Custom config (your own app schema) ===");
    let config = ParserConfig::new()
        .with_field("host", "text")
        .with_field("score", "number")
        .with_field("seen", "boolean")
        .with_field("genre", "multiSelect")
        .with_field("published", "date")
        .with_abbreviation("h", "host")
        .with_abbreviation("s", "score");

    let node = parse_query_with_config(
        "host:example.com score:>7 seen:true genre:fiction published:>2024-01-01",
        &config,
    ).unwrap();
    let structured = convert_to_structured(&node);
    println!("{}", serde_json::to_string_pretty(&structured).unwrap());
}
