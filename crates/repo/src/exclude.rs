//! `.atignore` exclude globs for repository discovery.
//!
//! An optional `.atignore` file at the analyzed root lists one pattern per
//! line (`#` comments and blank lines skipped, CRLF tolerated). Patterns are
//! gitignore-flavoured but deliberately small:
//!
//! - `*` and `?` match within one path segment; `**` matches zero or more
//!   whole segments.
//! - A pattern with no `/` matches an entry NAME at any depth.
//! - A pattern containing `/` is anchored at the root (a leading `/` is
//!   optional); a trailing `/**` also matches the directory itself.
//! - A trailing `/` restricts the pattern to directories.
//! - Negation (`!pattern`) is not supported; such lines are ignored.
//!
//! A matching directory is pruned from the walk, so nothing beneath it is
//! stat'd or hashed - that is where the time goes on vendored trees.

use std::fs;
use std::path::Path;

/// File name of the per-root exclude list.
pub const ATIGNORE_FILE_NAME: &str = ".atignore";

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExcludePattern {
    segments: Vec<String>,
    anchored: bool,
    directories_only: bool,
}

/// The parsed exclude list of one discovery root.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExcludeRules {
    patterns: Vec<ExcludePattern>,
}

impl ExcludeRules {
    /// Reads `<root>/.atignore`; a missing or unreadable file means no rules.
    pub fn load(root: &Path) -> Self {
        fs::read_to_string(root.join(ATIGNORE_FILE_NAME))
            .map(|text| Self::parse(&text))
            .unwrap_or_default()
    }

    /// Parses exclude-list text into rules.
    pub fn parse(text: &str) -> Self {
        let patterns = text.lines().filter_map(parse_pattern_line).collect();
        Self { patterns }
    }

    /// True when no pattern is configured (the common, zero-cost case).
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// Whether the root-relative, `/`-separated path should be skipped.
    pub fn is_excluded(&self, relative_path: &str, is_directory: bool) -> bool {
        let path_segments: Vec<&str> = relative_path.split('/').filter(|s| !s.is_empty()).collect();
        self.patterns
            .iter()
            .any(|pattern| pattern_matches(pattern, &path_segments, is_directory))
    }
}

fn parse_pattern_line(raw_line: &str) -> Option<ExcludePattern> {
    let line = raw_line.trim();
    if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
        return None;
    }
    let directories_only = line.ends_with('/');
    // A trailing `**` segment is kept: it matches zero segments too, so
    // `dir/**` covers the directory itself and every descendant.
    let body = line.trim_end_matches('/');
    let anchored = body.contains('/');
    let segments: Vec<String> = body
        .split('/')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    if segments.is_empty() {
        return None;
    }
    Some(ExcludePattern {
        segments,
        anchored,
        directories_only,
    })
}

fn pattern_matches(pattern: &ExcludePattern, path_segments: &[&str], is_directory: bool) -> bool {
    if pattern.directories_only && !is_directory {
        return false;
    }
    if !pattern.anchored {
        // Name pattern: matched against the entry's own name. Ancestors were
        // already tested (and pruned) when the walk visited them.
        return path_segments
            .last()
            .is_some_and(|name| segment_matches(&pattern.segments[0], name));
    }
    segments_match(&pattern.segments, path_segments)
}

/// Whole-path match where a `**` pattern segment spans zero or more segments.
fn segments_match(pattern_segments: &[String], path_segments: &[&str]) -> bool {
    match pattern_segments.split_first() {
        None => path_segments.is_empty(),
        Some((head, rest)) if head == "**" => {
            (0..=path_segments.len()).any(|skip| segments_match(rest, &path_segments[skip..]))
        }
        Some((head, rest)) => match path_segments.split_first() {
            Some((name, path_rest)) => {
                segment_matches(head, name) && segments_match(rest, path_rest)
            }
            None => false,
        },
    }
}

/// Single-segment glob: `*` = any run of characters, `?` = one character.
fn segment_matches(glob: &str, name: &str) -> bool {
    let glob_chars: Vec<char> = glob.chars().collect();
    let name_chars: Vec<char> = name.chars().collect();
    wildcard_match(&glob_chars, &name_chars)
}

fn wildcard_match(glob: &[char], name: &[char]) -> bool {
    match glob.split_first() {
        None => name.is_empty(),
        Some(('*', rest)) => (0..=name.len()).any(|skip| wildcard_match(rest, &name[skip..])),
        Some(('?', rest)) => !name.is_empty() && wildcard_match(rest, &name[1..]),
        Some((literal, rest)) => name.first() == Some(literal) && wildcard_match(rest, &name[1..]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comments_blank_lines_and_negations_are_skipped() {
        let rules = ExcludeRules::parse("# note\n\n   \n!keep.cpp\n");
        assert!(rules.is_empty());
    }

    #[test]
    fn name_pattern_matches_at_any_depth() {
        let rules = ExcludeRules::parse("*_test.cpp\n");
        assert!(rules.is_excluded("a/b/c/foo_test.cpp", false));
        assert!(rules.is_excluded("foo_test.cpp", false));
        assert!(!rules.is_excluded("a/foo_test.hpp", false));
    }

    #[test]
    fn anchored_pattern_needs_full_path_match() {
        let rules = ExcludeRules::parse("libs/*/test\n");
        assert!(rules.is_excluded("libs/system/test", true));
        assert!(!rules.is_excluded("x/libs/system/test", true));
        assert!(!rules.is_excluded("libs/system/test/extra", true));
    }

    #[test]
    fn leading_slash_is_optional_anchor() {
        let rules = ExcludeRules::parse("/doc\n");
        assert!(rules.is_excluded("doc", true));
    }

    #[test]
    fn trailing_double_star_matches_the_directory_itself() {
        let rules = ExcludeRules::parse("libs/*/test/**\n");
        assert!(rules.is_excluded("libs/system/test", true));
        assert!(rules.is_excluded("libs/system/test/a/b.cpp", false));
    }

    #[test]
    fn question_mark_matches_exactly_one_character() {
        let rules = ExcludeRules::parse("v?\n");
        assert!(rules.is_excluded("v1", true));
        assert!(!rules.is_excluded("v10", true));
        assert!(!rules.is_excluded("v", true));
    }

    #[test]
    fn missing_file_loads_as_empty_rules() {
        let dir = std::env::temp_dir().join("ariadne-atignore-missing-root-probe");
        assert!(ExcludeRules::load(&dir).is_empty());
    }
}
