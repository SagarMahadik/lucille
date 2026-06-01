# Lucille

**Rust parser for LLM-safe Lucene-style search syntax into a typed AST.**

Lucille parses compact, Lucene-inspired search queries into a typed, backend-neutral AST that applications can validate, inspect, and translate into their own query systems — without exposing raw SQL, MongoDB, or proprietary DSLs to untrusted input.

```rust
let node = lucille::parse_query("domain:google.com isFavorite:true")?;
let structured = lucille::convert_to_structured(&node);
// → typed, validated AST ready for translation
```

## Why Lucille?

### LLM-safe query generation

Instead of asking an LLM to generate SQL, MongoDB aggregations, or backend-specific query DSLs, ask it to emit **Lucille syntax** — a compact, well-defined search grammar that Lucille can parse, validate, and convert into a typed AST.

```
User request:       "show my favorite Rust articles from the last two years"
LLM generates:      type:article tag:rust isFavorite:true createdAt:>2024-01-01
Lucille parses:     ✅ validated AST (not raw SQL injection)
App translates:     → your query system (SQL / Elasticsearch / in-memory filter)
```

### Key benefits

- **No SQL injection** — the LLM can only emit structured filters, not arbitrary query code
- **Validation layer** — unknown fields, types, and malformed syntax are caught at parse time
- **Backend-neutral** — single AST translates to any query backend
- **Typed filters** — numbers, dates, booleans, multi-select, and text are distinguished in the AST

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

Custom abbreviations are easy to add via `ParserConfig::with_abbreviation()`.

### Custom properties

```
cp_author:Taleb
cp_isbn:9780143039433
```

## The AST (main product)

Lucille produces two representations of a parsed query. Both are fully typed, serializable, and backend-neutral.

### `SearchNode` — full binary AST

Represents the complete parse tree with logical operators as binary nodes:

| Variant | Purpose |
|---|---|
| `And { left, right }` | Logical AND |
| `Or { left, right }` | Logical OR |
| `Not { expression }` | Negation wrapper |
| `Filter { field, condition, value, field_type, is_custom_property }` | A single atomic filter |

### `StructuredQuery` — flattened query

Flattened form matching standard search API patterns:

| Variant | Shape |
|---|---|
| `Filter` | `{ field, condition, value, fieldType, isCustomProperty }` |
| `Group` | `{ operator: "and"\|"or", filters: [...] }` |
| `NotGroup` | `{ operator: "not", filter: {...} }` |

### Field types in the AST

| Type | Condition examples | Value type |
|---|---|---|
| `text` | `is`, `isNot`, `contains`, `startsWith`, `endsWith`, `isAnyOf`, `isNotAnyOf` | string or string[] |
| `boolean` | `"true"` / `"false"` (as string, matches JS output) | bool |
| `number` | `is`, `isNot`, `greaterThan`, `lessThan`, `isGreaterThanOrEqualTo`, `isLessThanOrEqualTo` | number |
| `date` | `is`, `before`, `after`, `onOrBefore`, `onOrAfter` | string (ISO date) |
| `multiSelect` | `includes`, `doesNotInclude`, `includeAnyOf`, `excludeAnyOf`, `includeAllOf`, `excludeAllOf` | string or string[] |

### Custom schema with `ParserConfig`

To use your own schema instead of the built-in defaults, build a `ParserConfig`:

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

You can also extend the built-in defaults instead of starting from scratch:

```rust
let config = ParserConfig::common_fields()
    .with_field("myCustomField", "number");
```

### ParserConfig methods

| Method | Description |
|---|---|
| `ParserConfig::new()` | Empty config — start from scratch |
| `ParserConfig::common_fields()` | Pre-populated with built-in fields and abbreviations |
| `.with_field(name, type)` | Register a field with its type |
| `.with_abbreviation(short, full)` | Register a shorthand alias |

## LLM-safe query generation (extended)

Lucille is designed to bridge LLMs and query systems safely.

### Pattern: validate-before-execute

```
LLM → Lucille syntax string → Lucille parser → typed AST → your query executor
```

The LLM never generates raw SQL, MongoDB JSON, or REST parameters. It emits a restricted grammar that Lucille validates before your app touches it.

### Example: RAG pipeline

```
User: "find my starred Python libraries from 2024"
LLM:  tag:python isFavorite:true createdAt:>=2024-01-01
Lucille: ✅ validates field types, parses date, produces typed AST
App:   → translates to SQL WHERE or vector store filter
```

### Example: multi-tenant search

```rust
// Per-tenant schema: each tenant has different searchable fields
let tenant_config = ParserConfig::new()
    .with_field("status", "multiSelect")
    .with_field("assignee", "text")
    .with_field("priority", "number");

// LLM generates tenant-aware query
let query = "status:open,active priority:>5 assignee:me";
let ast = parse_query_with_config(query, &tenant_config)?;
// Safely translate ast → tenant's query backend
```

## Full API reference

### Core functions

| Function | Description |
|---|---|
| `parse_query(input: &str) -> Option<SearchNode>` | Parse a search string into a raw AST |
| `parse_query_with_config(input, config) -> Option<SearchNode>` | Parse with custom field schema |
| `convert_to_structured(node: &SearchNode) -> Option<StructuredQuery>` | Transform AST into flattened structured query |
| `process_filter(property, value, is_negative, is_tag) -> Option<SearchNode>` | Process a single filter token (uses defaults) |
| `process_filter_with_config(property, value, is_negative, is_tag, config) -> Option<SearchNode>` | Process a single filter token with custom config |

### Tokenizer utilities

| Function | Description |
|---|---|
| `hybrid_tokenize(text: &str) -> String` | Lowercase + split on non-alphanumeric, join with spaces |
| `normalize_for_search(text: &str) -> String` | Lowercase only |
| `extract_note_text(note_json: &Value) -> String` | Flatten a structured note JSON to plain text |

## License

MIT
