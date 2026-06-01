# Lucille

A Lucene-like search query parser for Cherrypic, extracted as a standalone Rust library.

```rust
let node = lucille::parse_query("domain:google.com isFavorite:true")?;
let structured = lucille::convert_to_structured(&node);
```

## Quick Start

```toml
[dependencies]
lucille = "0.1.0"
serde_json = "1"
```

```rust
use lucille::{parse_query, convert_to_structured};

let result = parse_query("type:article rating:>3 tag:rust").unwrap();
let json = serde_json::to_string_pretty(&convert_to_structured(&result)).unwrap();
println!("{}", json);
```

Output:
```json
{
  "operator": "and",
  "filters": [
    { "field": "type", "condition": "is", "value": "article", "fieldType": "text", "isCustomProperty": false },
    { "field": "rating", "condition": "greaterThan", "value": 3.0, "fieldType": "number", "isCustomProperty": false },
    { "field": "tags", "condition": "includes", "value": "rust", "fieldType": "multiSelect", "isCustomProperty": false }
  ]
}
```

## Syntax

### Field filters

```
domain:google.com
type:article,book
rating:>3
createdAt:>=2024-01-01
```

### Tags

```
#rust
-#rust
tag:rust
tags:rust,typescript
```

### Negation

```
-domain:google.com
NOT domain:google.com
```

Both `-` prefix and `NOT` keyword produce the same result.

### Booleans

```
isFavorite:true
isFavorite:false
isArchived:0
```

### Logical operators

```
domain:google.com AND isFavorite:true
domain:google.com OR domain:twitter.com
(domain:google.com OR domain:twitter.com) AND isFavorite:true
```

Implicit AND is the default. Use `match:OR` to change the default to OR:
```
match:OR domain:google.com domain:twitter.com
```

### Wildcards

```
domain:*google*    # contains
domain:google*    # starts with
domain:*.com      # ends with
```

### Abbreviations

| Short | Resolves to |
|---|---|
| `t:` | `tags:` |
| `tag:` | `tags:` |
| `created:` | `createdAt:` |
| `modified:` | `updatedAt:` |

### Custom properties

```
cp_author:Taleb
cp_isbn:9780143039433
```

## API

| Function | Description |
|---|---|
| `parse_query(input: &str) -> Option<SearchNode>` | Parse a search string into a raw AST |
| `convert_to_structured(node: &SearchNode) -> Option<StructuredQuery>` | Transform AST into a flattened structured query |
| `process_filter(property, value, is_negative, is_tag) -> Option<SearchNode>` | Process a single filter token |
| `hybrid_tokenize(text: &str) -> String` | Tokenize text for FTS indexing |

### Custom schema with `ParserConfig`

By default Lucille ships with the Cherrypic field schema. To use it with your own schema, build a `ParserConfig`:

```rust
use lucille::{ParserConfig, parse_query_with_config, convert_to_structured};

let config = ParserConfig::new()
    .with_field("host", "text")
    .with_field("score", "number")
    .with_field("seen", "boolean")
    .with_field("genre", "multiSelect")
    .with_field("published", "date")
    .with_abbreviation("s", "score")
    .with_abbreviation("h", "host");

let result = parse_query_with_config(
    "host:example.com score:>7 seen:true genre:fiction h:local",
    &config,
)?;
```

You can also extend the Cherrypic defaults instead of starting from scratch:

```rust
let config = ParserConfig::cherrypic_defaults()
    .with_field("myCustomField", "number");
```

### Methods

| Method | Description |
|---|---|
| `ParserConfig::new()` | Empty config |
| `ParserConfig::cherrypic_defaults()` | Pre-populated with all Cherrypic fields/abbreviations |
| `.with_field(name, type)` | Register a field with its type |
| `.with_abbreviation(short, full)` | Register a shorthand alias |

### Supported field types

| Type | Behaviors |
|---|---|
| `text` | Tokenized match, wildcards `*`, comma-separated `isAnyOf`, FTS field mapping (`title`→`title_tokens`, etc.) |
| `boolean` | `true`/`false`/`0`/`1`, XOR negation |
| `number` | Operators `>`, `<`, `>=`, `<=`, bare numbers, negation inverts operator |
| `date` | Operators `>`, `<`, `>=`, `<=`, negation inverts operator |
| `multiSelect` | Comma-separated values, auto `includes`/`includeAnyOf`/`excludeAnyOf` |

## Output types

**`SearchNode`** — binary AST (`.type` = `AND` / `OR` / `NOT` / `FILTER`).

**`StructuredQuery`** — flattened format matching Cherrypic's JS parser output:
- `{ field, condition, value, fieldType, isCustomProperty }` — a single filter
- `{ operator: "and"|"or", filters: [...] }` — grouped filters
- `{ operator: "not", filter: {...} }` — negated group

### Supported field types

| Field type | Behavior |
|---|---|
| `text` | Tokenized FTS match, supports wildcards and comma-separated OR |
| `boolean` | `true` / `false` / `0` / `1`, XOR negation |
| `number` | Operators: `>`, `<`, `>=`, `<=`, `=` |
| `date` | Operators: `>`, `<`, `>=`, `<=` |
| `multiSelect` | Single or comma-separated values, `includes`/`includeAnyOf`/`excludeAnyOf` |

## License

MIT
