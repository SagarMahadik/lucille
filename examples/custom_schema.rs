/// Example: Custom parser config for a note-taking app.
///
/// Shows how to define a domain-specific schema with field types
/// and abbreviations tailored to your application.
use lucille::{ParserConfig, parse_query_with_config, convert_to_structured};

fn main() {
    let config = ParserConfig::new()
        .with_field("notebook", "text")
        .with_field("tags", "multiSelect")
        .with_field("wordCount", "number")
        .with_field("created", "date")
        .with_field("starred", "boolean")
        .with_abbreviation("nb", "notebook")
        .with_abbreviation("wc", "wordCount");

    let queries = [
        "notebook:recipes tag:pasta,italian",
        "wc:>500 created:>2024-01-01 starred:true",
        "nb:work tag:meeting created:>=2024-06-01",
    ];

    for q in &queries {
        println!("Query: {q}");
        let ast = parse_query_with_config(q, &config).expect("query should parse");
        println!("{}\n", serde_json::to_string_pretty(&convert_to_structured(&ast)).unwrap());
    }
}
