# LLM-safe query generation with Lucille

Lucille is designed for a specific pattern: **an LLM emits a search string, Lucille validates it, and your application translates the typed AST into a backend query.** This doc shows real query strings your LLM can generate and what Lucille produces from them.

## Pattern

```
User request → LLM → Lucille syntax string → Lucille parser → typed AST → your query executor
```

The LLM never emits SQL, MongoDB JSON, or Elasticsearch DSL. It only emits structured field filters that Lucille validates before your app touches them.

## Basic examples

### Single filter

LLM output:
```
type:article
```

Lucille AST:
```json
{ "field": "type", "condition": "is", "value": "article", "fieldType": "text" }
```

### Multiple filters (implicit AND)

LLM output:
```
type:article isFavorite:true
```

Lucille AST:
```json
{
  "operator": "and",
  "filters": [
    { "field": "type", "condition": "is", "value": "article", "fieldType": "text" },
    { "field": "isFavorite", "condition": "true", "value": true, "fieldType": "boolean" }
  ]
}
```

### Negation

LLM output:
```
type:article -tag:spam
```

Lucille AST:
```json
{
  "operator": "and",
  "filters": [
    { "field": "type", "condition": "is", "value": "article", "fieldType": "text" },
    { "field": "tags", "condition": "doesNotInclude", "value": "spam", "fieldType": "multiSelect" }
  ]
}
```

## Realistic RAG examples

### "Show my favorite Rust articles from the last two years"

LLM generates:
```
type:article tag:rust isFavorite:true createdAt:>2024-01-01
```

Lucille produces a 4-filter AND group with typed fields (`text`, `multiSelect`, `boolean`, `date`).

### "Find Python libraries starred in 2024 with high ratings"

LLM generates:
```
tag:python isFavorite:true createdAt:>=2024-01-01 rating:>4
```

### "Give me unread books by authors I follow, not in the TBR pile"

LLM generates:
```
type:book readStatus:1 -tag:tbr -tag:dnf
```

### "Active support tickets assigned to me with high priority"

LLM generates:
```
status:open,active assignee:me priority:>5
```

## Using abbreviations

LLM can use short aliases if your config registers them:

```
t:rust rating:>3 created:>2024-06-01
```

Resolves to:
```
tags:rust rating:>3 createdAt:>2024-06-01
```

## Per-tenant schema

Each tenant defines their own fields. The LLM is told which fields are available.

**Tenant A — Issue tracker:**
- `status` (multiSelect: open, closed, active)
- `assignee` (text)
- `priority` (number)

LLM generates: `status:active priority:>3`

**Tenant B — E-commerce:**
- `category` (multiSelect)
- `price` (number)
- `inStock` (boolean)

LLM generates: `category:electronics price:<500 inStock:true`

## Wildcards for partial matches

```
domain:*example*
title:rust*
description:*guide*
```

## Boolean logic with groups

```
(tag:rust OR tag:python) AND isFavorite:true
(domain:twitter.com OR domain:reddit.com) AND createdAt:>2024-01-01
```

## Batching LLM calls

Instead of one complex query, split into simpler ones:

1. `type:article tag:rust` — find candidates
2. `isFavorite:true rating:>4` — filter results

## Validation before execution

```rust
fn execute_llm_query(llm_output: &str, config: &ParserConfig) -> Result<(), String> {
    let ast = parse_query_with_config(llm_output, config)
        .ok_or("LLM output produced empty or invalid query")?;
    let structured = convert_to_structured(&ast)
        .ok_or("Failed to convert AST to structured query")?;
    // structured is now safe to translate to any backend
    translate_to_sql(&structured)?;
    Ok(())
}
```

## Prompt hint for LLMs

When asking an LLM to generate queries, include a brief spec:

```
Generate a search query using this grammar:
  field:value          — exact match on field
  field:>value         — greater than (numbers/dates)
  field:<value         — less than
  field:>=value        — on or after
  field:<=value        — on or before
  field:val1,val2      — match any of (comma = OR)
  field:*val*          — contains
  -field:value         — negation
  NOT field:value      — negation
  #tag                 — tag filter
  expr1 AND expr2      — boolean AND
  expr1 OR expr2       — boolean OR
  (expr)               — grouping

Available fields: type, tags, domain, isFavorite, rating, createdAt, ...
```

## See also

- [examples/](../examples/) — runnable Rust example files
- `examples/llm_rag_filter.rs` — LLM → Lucille → typed AST pipeline
- `examples/multi_tenant.rs` — per-tenant schema demo
- `examples/query_validation.rs` — validating LLM output before execution
