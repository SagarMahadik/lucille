use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::tokenizer;

#[derive(Debug, Clone)]
pub struct ParserConfig {
    pub field_types: HashMap<String, String>,
    pub abbreviations: HashMap<String, String>,
}

impl ParserConfig {
    pub fn new() -> Self {
        Self {
            field_types: HashMap::new(),
            abbreviations: HashMap::new(),
        }
    }

    pub fn with_field(mut self, name: &str, field_type: &str) -> Self {
        self.field_types.insert(name.to_string(), field_type.to_string());
        self
    }

    pub fn with_abbreviation(mut self, short: &str, full: &str) -> Self {
        self.abbreviations.insert(short.to_string(), full.to_string());
        self
    }

    pub fn cherrypic_defaults() -> Self {
        Self {
            field_types: cherrypic_field_types_raw(),
            abbreviations: cherrypic_abbreviations_raw(),
        }
    }
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self::cherrypic_defaults()
    }
}

fn cherrypic_field_types_raw() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("tags".to_string(), "multiSelect".to_string());
    m.insert("tag".to_string(), "multiSelect".to_string());
    m.insert("aiTags".to_string(), "multiSelect".to_string());
    m.insert("bookmarkId".to_string(), "text".to_string());
    m.insert("domain".to_string(), "text".to_string());
    m.insert("type".to_string(), "text".to_string());
    m.insert("objectType".to_string(), "text".to_string());
    m.insert("parent".to_string(), "text".to_string());
    m.insert("isFavorite".to_string(), "boolean".to_string());
    m.insert("isArchived".to_string(), "boolean".to_string());
    m.insert("isDeleted".to_string(), "boolean".to_string());
    m.insert("hasReminder".to_string(), "boolean".to_string());
    m.insert("withAnnotation".to_string(), "boolean".to_string());
    m.insert("withNotes".to_string(), "boolean".to_string());
    m.insert("withHighlights".to_string(), "boolean".to_string());
    m.insert("withBacklinks".to_string(), "boolean".to_string());
    m.insert("highlightLength".to_string(), "number".to_string());
    m.insert("rating".to_string(), "number".to_string());
    m.insert("readStatus".to_string(), "number".to_string());
    m.insert("status".to_string(), "number".to_string());
    m.insert("createdAt".to_string(), "date".to_string());
    m.insert("updatedAt".to_string(), "date".to_string());
    m.insert("time".to_string(), "date".to_string());
    m.insert("logDate".to_string(), "date".to_string());
    m.insert("logDueDate".to_string(), "date".to_string());
    m.insert("dueDate".to_string(), "date".to_string());
    m.insert("reminder_time".to_string(), "date".to_string());
    m.insert("fileSize".to_string(), "number".to_string());
    m.insert("duration".to_string(), "number".to_string());
    m
}

fn cherrypic_abbreviations_raw() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("t".to_string(), "tags".to_string());
    m.insert("tag".to_string(), "tags".to_string());
    m.insert("created".to_string(), "createdAt".to_string());
    m.insert("modified".to_string(), "updatedAt".to_string());
    m
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SearchNode {
    #[serde(rename = "AND")]
    And {
        left: Box<SearchNode>,
        right: Box<SearchNode>,
    },
    #[serde(rename = "OR")]
    Or {
        left: Box<SearchNode>,
        right: Box<SearchNode>,
    },
    #[serde(rename = "NOT")]
    Not { expression: Box<SearchNode> },
    #[serde(rename = "FILTER")]
    Filter {
        field: String,
        condition: String,
        value: Value,
        #[serde(rename = "fieldType")]
        field_type: String,
        #[serde(rename = "isCustomProperty")]
        is_custom_property: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    Filter,
    LogicalOperator,
    NotOperator,
    Metadata,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub property: Option<String>,
    pub value: String,
    pub is_negative: bool,
    pub is_tag: bool,
    pub default_operator: Option<String>,
}

pub struct Tokenizer {
    input: Vec<char>,
    position: usize,
    default_operator: String,
}

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        let mut input_str = input.to_string();
        let mut default_operator = "AND".to_string();

        let match_or_regex = Regex::new(r"(?i)\bmatch:OR\b").unwrap();
        if match_or_regex.is_match(&input_str) {
            default_operator = "OR".to_string();
            input_str = match_or_regex.replace_all(&input_str, " ").to_string();
        }

        Self {
            input: input_str.chars().collect(),
            position: 0,
            default_operator,
        }
    }

    fn is_eof(&self) -> bool {
        self.position >= self.input.len()
    }

    fn peek(&self, offset: usize) -> Option<char> {
        if self.position + offset < self.input.len() {
            Some(self.input[self.position + offset])
        } else {
            None
        }
    }

    fn consume(&mut self) -> Option<char> {
        if self.is_eof() {
            None
        } else {
            let ch = self.input[self.position];
            self.position += 1;
            Some(ch)
        }
    }

    fn skip_whitespace(&mut self) {
        while !self.is_eof() && self.peek(0).unwrap().is_whitespace() {
            self.consume();
        }
    }

    fn read_word_until_delimiter(&mut self) -> String {
        let mut word = String::new();
        while !self.is_eof() {
            let ch = self.peek(0).unwrap();
            if ch.is_whitespace() || ch == ')' || ch == ':' || ch == '"' || ch == '#' {
                break;
            }
            if ch.is_alphanumeric() || "_<>=+-.[],|/*".contains(ch) {
                word.push(self.consume().unwrap());
            } else {
                break;
            }
        }
        word
    }

    fn read_quoted_string(&mut self) -> String {
        self.consume(); // Skip "
        let mut value = String::new();
        while !self.is_eof() && self.peek(0).unwrap() != '"' {
            if self.peek(0).unwrap() == '\\' && self.peek(1) == Some('"') {
                self.consume(); // Skip \
            }
            value.push(self.consume().unwrap());
        }
        if !self.is_eof() && self.peek(0).unwrap() == '"' {
            self.consume(); // Skip closing "
        }
        value
    }

    fn read_until_logical_operator(&mut self) -> String {
        let mut value = String::new();
        let mut depth = 0;
        let mut in_quotes = false;

        while !self.is_eof() {
            let ch = self.peek(0).unwrap();

            if ch == '"' {
                in_quotes = !in_quotes;
                value.push(self.consume().unwrap());
                continue;
            }

            if !in_quotes {
                if ch == '(' {
                    depth += 1;
                    value.push(self.consume().unwrap());
                    continue;
                }
                if ch == ')' {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    value.push(self.consume().unwrap());
                    continue;
                }

                if depth == 0 && ch.is_whitespace() {
                    let mut lookahead = String::new();
                    let mut i = 0;
                    while let Some(c) = self.peek(i) {
                        if !c.is_whitespace() {
                            break;
                        }
                        i += 1;
                    }

                    let mut is_tag = false;
                    if self.peek(i) == Some('#')
                        || (self.peek(i) == Some('-') && self.peek(i + 1) == Some('#'))
                    {
                        is_tag = true;
                    }

                    let mut j = i;
                    let mut is_negative = false;
                    if self.peek(j) == Some('-') {
                        is_negative = true;
                        j += 1;
                        lookahead.push('-');
                    }

                    while let Some(c) = self.peek(j) {
                        if c.is_whitespace() || c == ')' || c == ':' || c == '#' {
                            break;
                        }
                        lookahead.push(c);
                        j += 1;
                    }

                    let upper = lookahead.to_uppercase();
                    if upper == "AND" || upper == "OR" {
                        break;
                    }

                    if self.peek(j) == Some(':') || is_tag {
                        break;
                    }
                }
            }

            if !in_quotes && depth == 0 && ":#".contains(ch) && !value.ends_with(':') {
                break;
            }

            value.push(self.consume().unwrap());
        }
        value.trim().to_string()
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while !self.is_eof() {
            self.skip_whitespace();
            if self.is_eof() {
                break;
            }

            let char = self.peek(0).unwrap();

            if char == '(' || char == ')' {
                tokens.push(Token {
                    token_type: if char == '(' {
                        TokenType::LeftParen
                    } else {
                        TokenType::RightParen
                    },
                    property: None,
                    value: char.to_string(),
                    is_negative: false,
                    is_tag: false,
                    default_operator: None,
                });
                self.consume();
                continue;
            }

            if char == '#' || (char == '-' && self.peek(1) == Some('#')) {
                let is_negative = char == '-';
                if is_negative {
                    self.consume();
                }
                self.consume(); // Skip #

                let mut tag_value = String::new();
                while !self.is_eof() {
                    let ch = self.peek(0).unwrap();
                    if ch.is_alphanumeric() || "_-/*".contains(ch) {
                        tag_value.push(self.consume().unwrap());
                    } else {
                        break;
                    }
                }
                tokens.push(Token {
                    token_type: TokenType::Filter,
                    property: Some("tags".to_string()),
                    value: tag_value,
                    is_negative,
                    is_tag: true,
                    default_operator: None,
                });
                continue;
            }

            if char == '-' {
                if self.peek(1) == Some('"') {
                    self.consume(); // Skip -
                    let val = self.read_quoted_string();
                    tokens.push(Token {
                        token_type: TokenType::Filter,
                        property: Some("content".to_string()),
                        value: format!("\"{}\"", val),
                        is_negative: true,
                        is_tag: false,
                        default_operator: None,
                    });
                    continue;
                } else if let Some(next) = self.peek(1) {
                    if next.is_alphanumeric() || next == '_' || next == '#' {
                        let mut lookahead_pos = self.position + 1;
                        let mut word = String::new();
                        while lookahead_pos < self.input.len() {
                            let c = self.input[lookahead_pos];
                            if c.is_whitespace() || "():\"#".contains(c) {
                                break;
                            }
                            word.push(c);
                            lookahead_pos += 1;
                        }
                        while lookahead_pos < self.input.len()
                            && self.input[lookahead_pos].is_whitespace()
                        {
                            lookahead_pos += 1;
                        }

                        let is_property =
                            lookahead_pos < self.input.len() && self.input[lookahead_pos] == ':';

                        if !is_property {
                            self.consume(); // Skip -
                            let val = self.read_word_until_delimiter();
                            if !val.is_empty() {
                                tokens.push(Token {
                                    token_type: TokenType::Filter,
                                    property: Some("content".to_string()),
                                    value: val,
                                    is_negative: true,
                                    is_tag: false,
                                    default_operator: None,
                                });
                            }
                            continue;
                        }
                    }
                }
            }

            if char == '"' {
                let val = self.read_quoted_string();
                tokens.push(Token {
                    token_type: TokenType::Filter,
                    property: Some("content".to_string()),
                    value: format!("\"{}\"", val),
                    is_negative: false,
                    is_tag: false,
                    default_operator: None,
                });
                continue;
            }

            let word = self.read_word_until_delimiter();
            if !word.is_empty() {
                let upper = word.to_uppercase();
                if upper == "AND" {
                    tokens.push(Token {
                        token_type: TokenType::LogicalOperator,
                        property: None,
                        value: "AND".to_string(),
                        is_negative: false,
                        is_tag: false,
                        default_operator: None,
                    });
                } else if upper == "OR" {
                    tokens.push(Token {
                        token_type: TokenType::LogicalOperator,
                        property: None,
                        value: "OR".to_string(),
                        is_negative: false,
                        is_tag: false,
                        default_operator: None,
                    });
                } else if upper == "NOT" {
                    self.skip_whitespace();
                    if !self.is_eof() {
                        tokens.push(Token {
                            token_type: TokenType::NotOperator,
                            property: None,
                            value: "NOT".to_string(),
                            is_negative: false,
                            is_tag: false,
                            default_operator: None,
                        });
                    } else {
                        tokens.push(Token {
                            token_type: TokenType::Filter,
                            property: Some("content".to_string()),
                            value: word,
                            is_negative: false,
                            is_tag: false,
                            default_operator: None,
                        });
                    }
                } else {
                    let mut is_property = false;
                    let mut temp_pos = self.position;
                    while temp_pos < self.input.len() && self.input[temp_pos].is_whitespace() {
                        temp_pos += 1;
                    }
                    if temp_pos < self.input.len() && self.input[temp_pos] == ':' {
                        self.position = temp_pos + 1;
                        is_property = true;
                    }

                    if is_property {
                        let property = word;
                        self.skip_whitespace();
                        let value = self.read_until_logical_operator();

                        let mut final_property = property;
                        let mut is_negative = false;
                        if final_property.starts_with('-') {
                            is_negative = true;
                            final_property = final_property[1..].to_string();
                        }

                        tokens.push(Token {
                            token_type: TokenType::Filter,
                            property: Some(final_property),
                            value,
                            is_negative,
                            is_tag: false,
                            default_operator: None,
                        });
                    } else {
                        tokens.push(Token {
                            token_type: TokenType::Filter,
                            property: Some("content".to_string()),
                            value: word,
                            is_negative: false,
                            is_tag: false,
                            default_operator: None,
                        });
                    }
                }
                continue;
            }

            if !self.is_eof() {
                self.consume();
            }
        }

        tokens.push(Token {
            token_type: TokenType::Metadata,
            property: None,
            value: String::new(),
            is_negative: false,
            is_tag: false,
            default_operator: Some(self.default_operator.clone()),
        });

        tokens
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    default_operator: String,
    config: ParserConfig,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self::with_config(tokens, ParserConfig::default())
    }

    pub fn with_config(tokens: Vec<Token>, config: ParserConfig) -> Self {
        let mut default_operator = "AND".to_string();
        let mut filtered_tokens = Vec::new();

        for token in tokens {
            if token.token_type == TokenType::Metadata {
                if let Some(op) = token.default_operator {
                    default_operator = op;
                }
            } else {
                filtered_tokens.push(token);
            }
        }

        Self {
            tokens: filtered_tokens,
            position: 0,
            default_operator,
            config,
        }
    }

    fn is_eof(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn consume(&mut self) -> Option<Token> {
        if self.is_eof() {
            None
        } else {
            let token = self.tokens[self.position].clone();
            self.position += 1;
            Some(token)
        }
    }

    fn check(&self, token_type: TokenType) -> bool {
        match self.peek() {
            Some(t) => t.token_type == token_type,
            None => false,
        }
    }

    pub fn parse(&mut self) -> Option<SearchNode> {
        if self.tokens.is_empty() {
            return None;
        }
        self.parse_expression()
    }

    fn parse_expression(&mut self) -> Option<SearchNode> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Option<SearchNode> {
        let mut left = self.parse_and()?;

        while !self.is_eof()
            && self.check(TokenType::LogicalOperator)
            && self.peek().unwrap().value == "OR"
        {
            self.consume(); // Consume OR
            if self.is_eof()
                || self.check(TokenType::LogicalOperator)
                || self.check(TokenType::RightParen)
            {
                break;
            }
            let right = self.parse_and()?;
            left = SearchNode::Or {
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Some(left)
    }

    fn parse_and(&mut self) -> Option<SearchNode> {
        let mut left = self.parse_unary()?;

        while !self.is_eof()
            && !self.check(TokenType::RightParen)
            && !self.check(TokenType::LogicalOperator)
        {
            let right = self.parse_unary()?;
            left = if self.default_operator == "OR" {
                SearchNode::Or {
                    left: Box::new(left),
                    right: Box::new(right),
                }
            } else {
                SearchNode::And {
                    left: Box::new(left),
                    right: Box::new(right),
                }
            };

            if !self.is_eof()
                && self.check(TokenType::LogicalOperator)
                && self.peek().unwrap().value == "OR"
            {
                break;
            }
        }

        while !self.is_eof()
            && self.check(TokenType::LogicalOperator)
            && self.peek().unwrap().value == "AND"
        {
            self.consume();
            if self.is_eof()
                || self.check(TokenType::LogicalOperator)
                || self.check(TokenType::RightParen)
            {
                break;
            }
            let right = self.parse_unary()?;
            left = SearchNode::And {
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Some(left)
    }

    fn parse_unary(&mut self) -> Option<SearchNode> {
        if self.is_eof() {
            return None;
        }

        if self.check(TokenType::NotOperator) {
            self.consume();
            if self.is_eof()
                || self.check(TokenType::LogicalOperator)
                || self.check(TokenType::RightParen)
            {
                return None;
            }
            let expr = self.parse_atom()?;
            return match expr {
                SearchNode::Filter {
                    field,
                    condition,
                    value,
                    field_type,
                    is_custom_property,
                } => {
                    Some(SearchNode::Not {
                        expression: Box::new(SearchNode::Filter {
                            field,
                            condition,
                            value,
                            field_type,
                            is_custom_property,
                        }),
                    })
                }
                SearchNode::Not { expression } => Some(*expression),
                _ => Some(SearchNode::Not {
                    expression: Box::new(expr),
                }),
            };
        }

        self.parse_atom()
    }

    fn parse_atom(&mut self) -> Option<SearchNode> {
        if self.is_eof() {
            return None;
        }

        if self.check(TokenType::Filter) {
            let token = self.consume().unwrap();
            let raw_property = token.property.unwrap_or_else(|| "content".to_string());

            return process_filter_with_config(
                &raw_property,
                &token.value,
                token.is_negative,
                token.is_tag,
                &self.config,
            );
        }

        if self.check(TokenType::LeftParen) {
            self.consume();
            let expr = self.parse_expression();
            if !self.is_eof() && self.check(TokenType::RightParen) {
                self.consume();
            }
            return expr;
        }

        self.consume();
        None
    }
}

pub fn process_filter(
    raw_property: &str,
    value: &str,
    is_negative_from_not: bool,
    is_tag: bool,
) -> Option<SearchNode> {
    process_filter_with_config(raw_property, value, is_negative_from_not, is_tag, &ParserConfig::default())
}

pub fn process_filter_with_config(
    raw_property: &str,
    value: &str,
    is_negative_from_not: bool,
    is_tag: bool,
    config: &ParserConfig,
) -> Option<SearchNode> {
    let lower_prop = raw_property.to_lowercase();
    let resolved_property = config
        .abbreviations
        .get(lower_prop.as_str())
        .cloned()
        .unwrap_or_else(|| raw_property.to_string());

    let (field, is_custom) = if resolved_property.starts_with("cp_") {
        (resolved_property[3..].to_string(), true)
    } else {
        (resolved_property, false)
    };

    let field_type = config
        .field_types
        .get(field.as_str())
        .cloned()
        .unwrap_or_else(|| "text".to_string());

    if is_tag {
        let condition = if is_negative_from_not {
            "doesNotInclude"
        } else {
            "includes"
        };
        return Some(SearchNode::Filter {
            field: "tags".to_string(),
            condition: condition.to_string(),
            value: Value::String(value.to_string()),
            field_type: "multiSelect".to_string(),
            is_custom_property: false,
        });
    }

    let mut condition = if is_negative_from_not { "isNot" } else { "is" };
    let mut final_value = value.to_string();

    if field_type == "text" || field_type == "searchTerm" || field == "content" {
        let fts_field = match field.as_str() {
            "title" => "title_tokens",
            "description" | "desc" => "desc_tokens",
            "url" => "url_tokens",
            "text" => "text_tokens",
            "note" => "note_tokens",
            "content" => "content_tokens",
            _ => field.as_str(),
        };

        if final_value.starts_with('*') && final_value.ends_with('*') {
            condition = if is_negative_from_not {
                "doesNotContain"
            } else {
                "contains"
            };
            final_value = if final_value.len() > 1 {
                final_value[1..final_value.len() - 1].to_string()
            } else {
                String::new()
            };
        } else if final_value.starts_with('*') {
            condition = if is_negative_from_not {
                "doesNotEndWith"
            } else {
                "endsWith"
            };
            final_value = final_value[1..].to_string();
        } else if final_value.ends_with('*') {
            condition = if is_negative_from_not {
                "doesNotStartWith"
            } else {
                "startsWith"
            };
            final_value = final_value[..final_value.len() - 1].to_string();
        } else if !fts_field.ends_with("_tokens") && final_value.contains(',') {
            let values: Vec<Value> = final_value
                .split(',')
                .map(|s| Value::String(s.trim().to_string()))
                .filter(|v| v.as_str().map(|s| !s.is_empty()).unwrap_or(false))
                .collect();

            if !values.is_empty() {
                return Some(SearchNode::Filter {
                    field,
                    condition: if is_negative_from_not {
                        "isNotAnyOf"
                    } else {
                        "isAnyOf"
                    }
                    .to_string(),
                    value: Value::Array(values),
                    field_type,
                    is_custom_property: is_custom,
                });
            }
        }

        let tokenized_value = tokenizer::hybrid_tokenize(&final_value);

        return Some(SearchNode::Filter {
            field: fts_field.to_string(),
            condition: condition.to_string(),
            value: Value::String(tokenized_value),
            field_type: field_type.to_string(),
            is_custom_property: is_custom,
        });
    } else if field_type == "boolean" {
        let is_false_val = final_value == "false" || final_value == "0";
        let final_is_neg = is_negative_from_not ^ is_false_val;
        return Some(SearchNode::Filter {
            field,
            condition: (!final_is_neg).to_string(),
            value: Value::Bool(!final_is_neg),
            field_type,
            is_custom_property: is_custom,
        });
    } else if field_type == "number" {
        let op_regex = Regex::new(r"^(?P<op>>=?|<=?)(?P<num>-?\d+(\.\d+)?)$").unwrap();
        if let Some(caps) = op_regex.captures(&final_value) {
            let op = caps.name("op").unwrap().as_str();
            let num: f64 = caps.name("num").unwrap().as_str().parse().unwrap_or(0.0);
            let mut cond = match op {
                "<" => "lessThan",
                ">" => "greaterThan",
                "<=" => "isLessThanOrEqualTo",
                ">=" => "isGreaterThanOrEqualTo",
                _ => "is",
            };

            if is_negative_from_not {
                cond = match cond {
                    "lessThan" => "isGreaterThanOrEqualTo",
                    "greaterThan" => "isLessThanOrEqualTo",
                    "isLessThanOrEqualTo" => "greaterThan",
                    "isGreaterThanOrEqualTo" => "lessThan",
                    _ => cond,
                };
            }
            return Some(SearchNode::Filter {
                field,
                condition: cond.to_string(),
                value: serde_json::to_value(num).unwrap(),
                field_type,
                is_custom_property: is_custom,
            });
        }

        if let Ok(num) = final_value.parse::<f64>() {
            return Some(SearchNode::Filter {
                field,
                condition: (if is_negative_from_not { "isNot" } else { "is" }).to_string(),
                value: serde_json::to_value(num).unwrap(),
                field_type,
                is_custom_property: is_custom,
            });
        }
    } else if field_type == "multiSelect" {
        let condition = if is_negative_from_not {
            "excludeAnyOf"
        } else {
            "includeAnyOf"
        };
        let values: Vec<Value> = value
            .split(',')
            .map(|s| Value::String(s.trim().to_string()))
            .collect();
        let final_condition = if values.len() == 1 {
            if is_negative_from_not {
                "doesNotInclude"
            } else {
                "includes"
            }
        } else {
            condition
        };
        return Some(SearchNode::Filter {
            field,
            condition: final_condition.to_string(),
            value: if values.len() == 1 {
                values[0].clone()
            } else {
                Value::Array(values)
            },
            field_type,
            is_custom_property: is_custom,
        });
    } else if field_type == "date" {
        let op_regex = Regex::new(r"^(?P<op>>=?|<=?)(?P<date>.+)$").unwrap();
        if let Some(caps) = op_regex.captures(&final_value) {
            let op = caps.name("op").unwrap().as_str();
            let date_str = caps.name("date").unwrap().as_str().to_string();
            let mut cond = match op {
                "<" => "before",
                ">" => "after",
                "<=" => "onOrBefore",
                ">=" => "onOrAfter",
                _ => "in",
            };
            if is_negative_from_not {
                cond = match cond {
                    "before" => "onOrAfter",
                    "after" => "onOrBefore",
                    "onOrBefore" => "after",
                    "onOrAfter" => "before",
                    _ => cond,
                };
            }
            return Some(SearchNode::Filter {
                field,
                condition: cond.to_string(),
                value: Value::String(date_str),
                field_type,
                is_custom_property: is_custom,
            });
        }
    }

    Some(SearchNode::Filter {
        field,
        condition: condition.to_string(),
        value: Value::String(final_value),
        field_type,
        is_custom_property: is_custom,
    })
}

fn is_pure_content_and_tree(node: &SearchNode) -> bool {
    match node {
        SearchNode::Filter {
            field, condition, ..
        } => field == "content_tokens" && !condition.to_lowercase().contains("not"),
        SearchNode::And { left, right } => {
            is_pure_content_and_tree(left) && is_pure_content_and_tree(right)
        }
        _ => false,
    }
}

fn contains_and(node: &SearchNode) -> bool {
    match node {
        SearchNode::And { .. } => true,
        SearchNode::Or { left, right } => contains_and(left) || contains_and(right),
        SearchNode::Not { expression } => contains_and(expression),
        _ => false,
    }
}

fn convert_and_to_or(node: SearchNode) -> SearchNode {
    match node {
        SearchNode::And { left, right } => SearchNode::Or {
            left: Box::new(convert_and_to_or(*left)),
            right: Box::new(convert_and_to_or(*right)),
        },
        SearchNode::Or { left, right } => SearchNode::Or {
            left: Box::new(convert_and_to_or(*left)),
            right: Box::new(convert_and_to_or(*right)),
        },
        SearchNode::Not { expression } => SearchNode::Not {
            expression: Box::new(convert_and_to_or(*expression)),
        },
        _ => node,
    }
}

pub fn parse_query(input: &str) -> Option<SearchNode> {
    parse_query_with_config(input, &ParserConfig::default())
}

pub fn parse_query_with_config(input: &str, config: &ParserConfig) -> Option<SearchNode> {
    let mut tokenizer = Tokenizer::new(input);
    let tokens = tokenizer.tokenize();
    let mut parser = Parser::with_config(tokens, config.clone());
    let mut ast = parser.parse();

    if let Some(node) = ast.take() {
        if contains_and(&node) && is_pure_content_and_tree(&node) {
            ast = Some(convert_and_to_or(node));
        } else {
            ast = Some(node);
        }
    }

    ast
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StructuredQuery {
    Filter {
        field: String,
        condition: String,
        value: Value,
        #[serde(rename = "fieldType")]
        field_type: String,
        #[serde(rename = "isCustomProperty")]
        is_custom_property: bool,
    },
    Group {
        operator: String,
        filters: Vec<StructuredQuery>,
    },
    NotGroup {
        operator: String,
        #[serde(rename = "filter")]
        filter: Box<StructuredQuery>,
    },
}

fn negate_condition(condition: &str, field_type: &str) -> String {
    if field_type == "boolean" {
        return if condition == "true" {
            "false".to_string()
        } else {
            "true".to_string()
        };
    }
    match condition {
        "is" => "isNot",
        "isNot" => "is",
        "contains" => "doesNotContain",
        "doesNotContain" => "contains",
        "containsAnyOf" => "doesNotContainAnyOf",
        "doesNotContainAnyOf" => "containsAnyOf",
        "endsWith" => "doesNotEndWith",
        "doesNotEndWith" => "endsWith",
        "startsWith" => "doesNotStartWith",
        "doesNotStartWith" => "startsWith",
        "isAnyOf" => "isNotAnyOf",
        "isNotAnyOf" => "isAnyOf",
        "greaterThan" => "isLessThanOrEqualTo",
        "isLessThanOrEqualTo" => "greaterThan",
        "lessThan" => "isGreaterThanOrEqualTo",
        "isGreaterThanOrEqualTo" => "lessThan",
        "includes" => "doesNotInclude",
        "doesNotInclude" => "includes",
        "includeAnyOf" => "excludeAnyOf",
        "excludeAnyOf" => "includeAnyOf",
        "includeAllOf" => "excludeAllOf",
        "excludeAllOf" => "includeAllOf",
        "before" => "onOrAfter",
        "after" => "onOrBefore",
        "onOrBefore" => "after",
        "onOrAfter" => "before",
        "in" => "notIn",
        "notIn" => "in",
        "between" => "notBetween",
        "notBetween" => "between",
        "isBetween" => "isNotBetween",
        "isNotBetween" => "isBetween",
        _ => condition,
    }
    .to_string()
}

pub fn convert_to_structured(node: &SearchNode) -> Option<StructuredQuery> {
    match node {
        SearchNode::Filter {
            field,
            condition,
            value,
            field_type,
            is_custom_property,
        } => Some(StructuredQuery::Filter {
            field: field.clone(),
            condition: condition.clone(),
            value: value.clone(),
            field_type: field_type.clone(),
            is_custom_property: *is_custom_property,
        }),
        SearchNode::Not { expression } => match expression.as_ref() {
            SearchNode::Filter {
                field,
                condition,
                value,
                field_type,
                is_custom_property,
            } => {
                let new_condition = negate_condition(condition, field_type);
                let new_value = if field_type == "boolean" {
                    Value::Bool(!value.as_bool().unwrap_or(false))
                } else {
                    value.clone()
                };
                Some(StructuredQuery::Filter {
                    field: field.clone(),
                    condition: new_condition,
                    value: new_value,
                    field_type: field_type.clone(),
                    is_custom_property: *is_custom_property,
                })
            }
            SearchNode::Not { expression: inner } => {
                // NOT (NOT x) -> x
                convert_to_structured(inner)
            }
            other => {
                let inner = convert_to_structured(other)?;
                Some(StructuredQuery::NotGroup {
                    operator: "not".to_string(),
                    filter: Box::new(inner),
                })
            }
        },
        SearchNode::And { left, right } | SearchNode::Or { left, right } => {
            let op = match node {
                SearchNode::And { .. } => "and",
                _ => "or",
            };
            let left_conv = convert_to_structured(left);
            let right_conv = convert_to_structured(right);

            let mut filters: Vec<StructuredQuery> = Vec::new();

            for child in [left_conv, right_conv].into_iter().flatten() {
                match child {
                    StructuredQuery::Group {
                        operator: ref child_op,
                        filters: ref sub_filters,
                    } if child_op == op => {
                        filters.extend(sub_filters.iter().cloned());
                    }
                    other => {
                        filters.push(other);
                    }
                }
            }

            let valid_filters: Vec<StructuredQuery> = filters;

            if valid_filters.is_empty() {
                return None;
            }
            if valid_filters.len() == 1 {
                return Some(valid_filters.into_iter().next().unwrap());
            }

            Some(StructuredQuery::Group {
                operator: op.to_string(),
                filters: valid_filters,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_filter() {
        let node = parse_query("domain:google.com").unwrap();
        if let SearchNode::Filter {
            field,
            condition,
            value,
            ..
        } = node
        {
            assert_eq!(field, "domain");
            assert_eq!(condition, "is");
            assert_eq!(value.as_str().unwrap(), "google.com");
        } else {
            panic!("Expected Filter node");
        }
    }

    #[test]
    fn test_and_logic() {
        let node = parse_query("tag:rust AND tag:ts").unwrap();
        if let SearchNode::And { left, right } = node {
            if let SearchNode::Filter { field, .. } = *left {
                assert_eq!(field, "tags");
            }
            if let SearchNode::Filter { field, .. } = *right {
                assert_eq!(field, "tags");
            }
        } else {
            panic!("Expected And node");
        }
    }

    #[test]
    fn test_and_to_or_conversion() {
        let node = parse_query("steve jobs").unwrap();
        if let SearchNode::Or { left, right } = node {
            if let SearchNode::Filter { field, value, .. } = *left {
                assert_eq!(field, "content_tokens");
                assert_eq!(value.as_str().unwrap(), "steve");
            } else {
                panic!("Expected Filter for left");
            }
            if let SearchNode::Filter { field, value, .. } = *right {
                assert_eq!(field, "content_tokens");
                assert_eq!(value.as_str().unwrap(), "jobs");
            } else {
                panic!("Expected Filter for right");
            }
        } else {
            panic!("Expected Or tree conversion, but got {:?}", node);
        }
    }

    #[test]
    fn test_negation() {
        let node = parse_query("-domain:google.com").unwrap();
        if let SearchNode::Filter {
            field, condition, ..
        } = node
        {
            assert_eq!(field, "domain");
            assert_eq!(condition, "isNot");
        } else {
            panic!("Expected Filter node");
        }
    }

    #[test]
    fn test_abbreviations() {
        let node = parse_query("t:rust").unwrap();
        if let SearchNode::Filter { field, .. } = node {
            assert_eq!(field, "tags");
        } else {
            panic!("Expected Filter node");
        }
    }

    #[test]
    fn test_partial_match() {
        let node = parse_query("domain:*google*").unwrap();
        if let SearchNode::Filter {
            condition, value, ..
        } = node
        {
            assert_eq!(condition, "contains");
            assert_eq!(value.as_str().unwrap(), "google");
        } else {
            panic!("Expected Filter node");
        }
    }

    #[test]
    fn test_multiple_filters() {
        let mut _tokenizer = Tokenizer::new("isFavorite:true domain:twitter.com");
        let node = parse_query("isFavorite:true domain:twitter.com").unwrap();
        println!("AST for multiple filters: {:#?}", node);
    }
}
