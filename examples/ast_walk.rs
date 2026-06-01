/// Example: Walking the full binary AST (SearchNode).
///
/// Lucille's raw AST preserves the exact parse tree with logical operators.
/// This is useful for query analysis, rewriting, or custom translation.
use lucille::{parse_query, SearchNode};

fn print_ast(node: &SearchNode, indent: usize) {
    let pad = "  ".repeat(indent);
    match node {
        SearchNode::And { left, right } => {
            println!("{pad}AND:");
            print_ast(left, indent + 1);
            print_ast(right, indent + 1);
        }
        SearchNode::Or { left, right } => {
            println!("{pad}OR:");
            print_ast(left, indent + 1);
            print_ast(right, indent + 1);
        }
        SearchNode::Not { expression } => {
            println!("{pad}NOT:");
            print_ast(expression, indent + 1);
        }
        SearchNode::Filter { field, condition, value, field_type, .. } => {
            println!("{pad}{field} {condition} {value} ({field_type})");
        }
    }
}

fn main() {
    let query = "(domain:google.com OR domain:twitter.com) AND isFavorite:true NOT tag:spam";
    let ast = parse_query(query).expect("should parse");
    println!("Query: {query}");
    println!("Binary AST:");
    print_ast(&ast, 0);
}
