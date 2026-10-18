//! Verbatim declaration-block locator for C++ sources (plan M23, D-M23-1).
//!
//! [`locate`] slices declaration text out of a source file by matching
//! symbol-name occurrences in the tokenizer stream. Comment, block-comment
//! and string-literal contents never reach the token stream, and `#if 0`
//! conditional regions are masked before matching, so occurrences inside
//! those regions never match. Names that cannot be located are omitted
//! from the result; the isolation engine keeps its placeholder body for
//! them (invariant I4).
//!
//! Name matching is deliberately simple (locked by the byte-exact tests):
//! the source-side qualified-name chain and the wanted name must be equal,
//! or one must be a segment-suffix of the other (`Thing` answers a
//! `Thing::Thing` constructor lookup; `Other::get_id` answers nothing for
//! `Thing::get_id`).

use crate::tokenizer::{tokenize_raw, RawToken};
use ariadne_lang::{SymbolBody, SymbolKind};

/// Locate verbatim declaration blocks for `wanted` inside `source`.
///
/// Results follow the order of `wanted`; each `SymbolBody::name` echoes the
/// wanted qualified name so the caller can map results back to its list.
pub(crate) fn locate(source: &str, wanted: &[(String, SymbolKind)]) -> Vec<SymbolBody> {
    let view = SourceView::new(source);
    wanted
        .iter()
        .filter_map(|(name, kind)| view.find_body(name, *kind))
        .collect()
}

/// Words that may never be absorbed into a declaration's left-hand prefix.
const PREFIX_STOP_WORDS: &[&str] = &[
    "case", "class", "default", "do", "else", "enum", "for", "goto", "if", "namespace",
    "new", "return", "struct", "switch", "throw", "typedef", "union", "using", "while",
];

/// Punctuation that may be absorbed into a declaration's left-hand prefix
/// (return-type decorations and template argument brackets).
const PREFIX_PUNCT: &[char] = &['*', '&', '~', '<', '>'];

/// A tokenized source with its `#if 0`-masked token indices precomputed.
struct SourceView {
    chars: Vec<char>,
    tokens: Vec<RawToken>,
    masked: Vec<bool>,
}

impl SourceView {
    fn new(source: &str) -> Self {
        let tokens = tokenize_raw(source);
        let masked = masked_region(&tokens);
        Self {
            chars: source.chars().collect(),
            tokens,
            masked,
        }
    }

    fn word(&self, index: usize) -> Option<&str> {
        self.tokens
            .get(index)
            .and_then(|t| match &t.kind {
                crate::tokenizer::TokenKind::Word(w) => Some(w.as_str()),
                _ => None,
            })
    }

    fn punct(&self, index: usize) -> Option<&str> {
        self.tokens
            .get(index)
            .and_then(|t| match &t.kind {
                crate::tokenizer::TokenKind::Punct(p) => Some(p.as_str()),
                _ => None,
            })
    }

    fn is_masked(&self, index: usize) -> bool {
        self.masked.get(index).copied().unwrap_or(false)
    }

    /// Verbatim text from the first char of token `start` through the last
    /// char of token `end` (both inclusive).
    fn slice(&self, start: usize, end: usize) -> String {
        let from = self.tokens[start].start;
        let to = self.tokens[end].end;
        self.chars[from..to].iter().collect()
    }

    fn body(&self, wanted: &str, start: usize, end: usize) -> SymbolBody {
        SymbolBody {
            name: wanted.to_string(),
            body: self.slice(start, end),
        }
    }

    /// Qualified name `Word (:: Word)*` starting at `index`.
    fn chain_after(&self, index: usize) -> Option<(Vec<String>, usize)> {
        let first = self.word(index)?;
        let mut segments = vec![first.to_string()];
        let mut next = index + 1;
        while self.punct(next) == Some(":")
            && self.punct(next + 1) == Some(":")
            && self.word(next + 2).is_some()
        {
            segments.push(self.word(next + 2).expect("checked").to_string());
            next += 3;
        }
        Some((segments, next))
    }

    /// Qualified name ending at `index` (the identifier before `(`).
    fn chain_before(&self, index: usize) -> Option<(Vec<String>, usize)> {
        let last = self.word(index)?;
        let mut segments = vec![last.to_string()];
        let mut start = index;
        while start >= 3
            && self.punct(start - 1) == Some(":")
            && self.punct(start - 2) == Some(":")
            && self.word(start - 3).is_some()
        {
            segments.insert(0, self.word(start - 3).expect("checked").to_string());
            start -= 3;
        }
        Some((segments, start))
    }

    fn find_body(&self, wanted: &str, kind: SymbolKind) -> Option<SymbolBody> {
        match kind {
            SymbolKind::Class | SymbolKind::Struct => self.find_record(wanted, &["class", "struct"]),
            SymbolKind::Union => self.find_record(wanted, &["union"]),
            SymbolKind::Enum => self.find_enum(wanted),
            SymbolKind::Function => self.find_function(wanted),
            SymbolKind::TypeAlias => self.find_type_alias(wanted),
            _ => None,
        }
    }

    /// `class`/`struct`/`union`/`enum` record: keyword through the matched
    /// `}` plus its trailing `;`, skipping brace-less forward declarations.
    fn find_record(&self, wanted: &str, keywords: &[&str]) -> Option<SymbolBody> {
        for i in 0..self.tokens.len() {
            if self.is_masked(i) || !keywords.contains(&self.word(i).unwrap_or("")) {
                continue;
            }
            if self.word(i.wrapping_sub(1)) == Some("enum") {
                continue; // `enum class X` belongs to the enum path
            }
            let Some((segments, after_chain)) = self.chain_after(i + 1) else {
                continue;
            };
            if !qualified_suffix_matches(wanted, &segments) {
                continue;
            }
            if let Some(end) = self.scan_record_body(after_chain) {
                return Some(self.body(wanted, self.extend_with_template(i), end));
            }
        }
        None
    }

    /// `enum [class] Name { ... };` — the slice starts at `enum`.
    fn find_enum(&self, wanted: &str) -> Option<SymbolBody> {
        for i in 0..self.tokens.len() {
            if self.is_masked(i) || self.word(i) != Some("enum") {
                continue;
            }
            let name_index = if matches!(self.word(i + 1), Some("class") | Some("struct")) {
                i + 2
            } else {
                i + 1
            };
            let Some((segments, after_chain)) = self.chain_after(name_index) else {
                continue;
            };
            if !qualified_suffix_matches(wanted, &segments) {
                continue;
            }
            if let Some(end) = self.scan_record_body(after_chain) {
                return Some(self.body(wanted, self.extend_with_template(i), end));
            }
        }
        None
    }

    /// From just after a record's name: find the first `{` (before any
    /// top-level `;`), match its braces, and include one trailing `;`.
    fn scan_record_body(&self, after_chain: usize) -> Option<usize> {
        let mut open = None;
        let mut index = after_chain;
        while index < self.tokens.len() {
            match self.punct(index) {
                Some("{") => {
                    open = Some(index);
                    break;
                }
                Some(";") => return None, // forward declaration
                _ => index += 1,
            }
        }
        let open = open?;
        let close = self.match_braces(open)?;
        let end = if self.punct(close + 1) == Some(";") {
            close + 1
        } else {
            close
        };
        Some(end)
    }

    /// Function declarations and definitions: return type (or `template`
    /// prefix / constructor name) through the matched body's `}` or the
    /// terminating `;`. Call sites never match: a declaration needs either
    /// a left-hand prefix or a constructor-shaped wanted name.
    fn find_function(&self, wanted: &str) -> Option<SymbolBody> {
        for i in 0..self.tokens.len() {
            if self.is_masked(i) || self.punct(i + 1) != Some("(") {
                continue;
            }
            let Some((segments, chain_start)) = self.chain_before(i) else {
                continue;
            };
            if !qualified_suffix_matches(wanted, &segments) {
                continue;
            }
            let prefix_start = self.walk_declaration_prefix(chain_start);
            let prefixless = prefix_start == chain_start;
            if prefixless && !looks_like_constructor(wanted) {
                continue; // bare call site or macro invocation
            }
            let start = self.extend_with_template(prefix_start);
            let Some(end) = self.scan_function_tail(i) else {
                continue;
            };
            return Some(self.body(wanted, start, end));
        }
        None
    }

    /// Walk left from the qualified-name start over same-line return-type
    /// tokens (`int`, `std::vector<int>`, `*`, `explicit`, ...).
    fn walk_declaration_prefix(&self, chain_start: usize) -> usize {
        let line = self.tokens[chain_start].line;
        let mut start = chain_start;
        while start > 0 {
            let previous = start - 1;
            if self.tokens[previous].line != line {
                break;
            }
            if let Some(word) = self.word(previous) {
                if PREFIX_STOP_WORDS.contains(&word) {
                    break;
                }
                start = previous;
                continue;
            }
            match self.punct(previous) {
                Some(":") if previous >= 1 && self.punct(previous - 1) == Some(":") => {
                    start = previous - 1; // consume the `::` pair
                }
                Some(p)
                    if p.chars().next().map(|c| PREFIX_PUNCT.contains(&c)) == Some(true) =>
                {
                    start = previous;
                }
                _ => break,
            }
        }
        // A slice may not start on stray punctuation (e.g. the `>` of a
        // dangling template prefix from a previous construct).
        if self.word(start).is_none() && self.punct(start) != Some("~") {
            return chain_start;
        }
        start
    }

    /// After the candidate's `(...)`: end at the first `{`-delimited body
    /// (definition) or the first `;` (declaration, includes `= 0;`,
    /// `= delete;`, `const;` and constructor init-lists on the way).
    fn scan_function_tail(&self, name_index: usize) -> Option<usize> {
        let close = self.match_parens(name_index + 1)?;
        let mut index = close + 1;
        while index < self.tokens.len() {
            if self.punct(index) == Some(";") {
                return Some(index);
            }
            if self.punct(index) == Some("{") {
                return self.match_braces(index);
            }
            index += 1;
        }
        None
    }

    /// `typedef ...;` and `using Name = ...;` through their `;`.
    fn find_type_alias(&self, wanted: &str) -> Option<SymbolBody> {
        for i in 0..self.tokens.len() {
            if self.is_masked(i) || !self.starts_top_level_statement(i) {
                continue;
            }
            if self.word(i) == Some("typedef") {
                if let Some(end) = self.scan_to_semicolon(i + 1) {
                    if self.region_holds_name(i, end, wanted) {
                        return Some(self.body(wanted, i, end));
                    }
                }
            } else if self.word(i) == Some("using") && self.word(i + 1) != Some("namespace") {
                let Some((segments, _)) = self.chain_after(i + 1) else {
                    continue;
                };
                if !qualified_suffix_matches(wanted, &segments) {
                    continue;
                }
                if let Some(end) = self.scan_to_semicolon(i + 1) {
                    return Some(self.body(wanted, self.extend_with_template(i), end));
                }
            }
        }
        None
    }

    fn starts_top_level_statement(&self, index: usize) -> bool {
        if index == 0 {
            return true;
        }
        self.tokens[index - 1].line != self.tokens[index].line
            || matches!(self.punct(index - 1), Some(";") | Some("}") | Some("{") | Some("#"))
    }

    /// Any qualified-name chain inside `(start, end]` that matches `wanted`.
    fn region_holds_name(&self, start: usize, end: usize, wanted: &str) -> bool {
        (start + 1..end).any(|j| {
            self.word(j).is_some()
                && self
                    .chain_after(j)
                    .filter(|(_, next)| *next <= end)
                    .map(|(segments, _)| qualified_suffix_matches(wanted, &segments))
                    .unwrap_or(false)
        })
    }

    fn scan_to_semicolon(&self, from: usize) -> Option<usize> {
        let mut depth = 0usize;
        let mut index = from;
        while index < self.tokens.len() {
            match self.punct(index) {
                Some("{") => depth += 1,
                Some("}") => depth = depth.saturating_sub(1),
                Some(";") if depth == 0 => return Some(index),
                _ => {}
            }
            index += 1;
        }
        None
    }

    /// Extend a slice start leftward over an immediately preceding
    /// `template < ... >` group.
    fn extend_with_template(&self, start: usize) -> usize {
        if start == 0 || self.punct(start - 1) != Some(">") {
            return start;
        }
        let mut depth = 1usize;
        let mut probe = start - 1;
        while probe > 0 {
            probe -= 1;
            match self.punct(probe) {
                Some(">") => depth += 1,
                Some("<") => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
        }
        if depth == 0 && probe > 0 && self.word(probe - 1) == Some("template") {
            probe - 1
        } else {
            start
        }
    }

    fn match_braces(&self, open: usize) -> Option<usize> {
        self.match_group(open, "{", "}")
    }

    fn match_parens(&self, open: usize) -> Option<usize> {
        self.match_group(open, "(", ")")
    }

    fn match_group(&self, open: usize, opener: &str, closer: &str) -> Option<usize> {
        if self.punct(open) != Some(opener) {
            return None;
        }
        let mut depth = 0usize;
        let mut index = open;
        while index < self.tokens.len() {
            match self.punct(index) {
                Some(p) if p == opener => depth += 1,
                Some(p) if p == closer => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(index);
                    }
                }
                _ => {}
            }
            index += 1;
        }
        None
    }
}

/// True when the wanted qualified name and the source-side name chain are
/// equal, or one is a segment-suffix of the other.
fn qualified_suffix_matches(wanted: &str, chain: &[String]) -> bool {
    let wanted_segments: Vec<&str> = wanted.split("::").collect();
    let chain_segments: Vec<&str> = chain.iter().map(String::as_str).collect();
    (chain_segments.len() <= wanted_segments.len() && wanted_segments.ends_with(&chain_segments))
        || (wanted_segments.len() <= chain_segments.len()
            && chain_segments.ends_with(&wanted_segments))
}

/// `X::X` (or deeper `n::X::X`): the wanted name addresses a constructor,
/// which may legitimately be declared with no return-type prefix.
fn looks_like_constructor(wanted: &str) -> bool {
    let segments: Vec<&str> = wanted.split("::").collect();
    segments.len() >= 2 && segments[segments.len() - 1] == segments[segments.len() - 2]
}

/// Token mask for `#if 0` (and nested) conditional regions: `true` for
/// tokens inside a region the compiler would never see. Only literal
/// `#if 0` conditions mask; `#elif` conservatively masks its branch.
fn masked_region(tokens: &[RawToken]) -> Vec<bool> {
    let mut masked = vec![false; tokens.len()];
    let mut conditions: Vec<bool> = Vec::new();
    for index in 0..tokens.len() {
        if is_directive_start(tokens, index) {
            apply_directive(&mut conditions, tokens, index);
        }
        masked[index] = conditions.contains(&true);
    }
    masked
}

fn is_directive_start(tokens: &[RawToken], index: usize) -> bool {
    let is_hash = match &tokens[index].kind {
        crate::tokenizer::TokenKind::Punct(p) => p == "#",
        _ => false,
    };
    is_hash && (index == 0 || tokens[index - 1].line != tokens[index].line)
}

fn apply_directive(conditions: &mut Vec<bool>, tokens: &[RawToken], hash_index: usize) {
    let directive = word_at(tokens, hash_index + 1).unwrap_or("");
    let parent_disabled = conditions.contains(&true);
    match directive {
        "if" => {
            let zero_literal = word_at(tokens, hash_index + 2) == Some("0");
            conditions.push(parent_disabled || zero_literal);
        }
        "ifdef" | "ifndef" => conditions.push(parent_disabled),
        "else" | "elif" => {
            if let Some(top) = conditions.last_mut() {
                *top = if directive == "else" { parent_disabled } else { true };
            }
        }
        "endif" => {
            conditions.pop();
        }
        _ => {}
    }
}

fn word_at<'t>(tokens: &'t [RawToken], index: usize) -> Option<&'t str> {
    match tokens.get(index).map(|t| &t.kind) {
        Some(crate::tokenizer::TokenKind::Word(w)) => Some(w.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod scan_branch_tests;
