//! Textual `#include` directive parser (plan M18.1: relocated verbatim from
//! `crates/graph/src/resolver.rs`; §6.2: port of every existing test case).

/// Parsed raw include directive from source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawIncludeDirective {
    pub raw_path: String,
    pub is_angled: bool,
    pub line: u32,
    pub column: u32,
}

/// Helper to parse `#include` lines from C/C++ source/header text.
pub fn parse_include_directives(content: &str) -> Vec<RawIncludeDirective> {
    let mut directives = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('#') {
            continue;
        }

        let after_hash = trimmed[1..].trim_start();
        if !after_hash.starts_with("include") {
            continue;
        }

        let after_include = after_hash["include".len()..].trim_start();
        let (raw_path, is_angled, col_offset) = if after_include.starts_with('"') {
            if let Some(end_idx) = after_include[1..].find('"') {
                let path = &after_include[1..=end_idx];
                (path.to_string(), false, line.find('"').unwrap_or(0) + 1)
            } else {
                continue;
            }
        } else if after_include.starts_with('<') {
            if let Some(end_idx) = after_include[1..].find('>') {
                let path = &after_include[1..=end_idx];
                (path.to_string(), true, line.find('<').unwrap_or(0) + 1)
            } else {
                continue;
            }
        } else {
            continue;
        };

        directives.push(RawIncludeDirective {
            raw_path,
            is_angled,
            line: (line_idx + 1) as u32,
            column: col_offset as u32,
        });
    }

    directives
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_include_directives() {
        let content = r#"
#pragma once
#include "thing_types.h"
#  include <vector>
#include "heavy.h" // comment
"#;

        let directives = parse_include_directives(content);
        assert_eq!(directives.len(), 3);
        assert_eq!(directives[0].raw_path, "thing_types.h");
        assert!(!directives[0].is_angled);
        assert_eq!(directives[0].line, 3);

        assert_eq!(directives[1].raw_path, "vector");
        assert!(directives[1].is_angled);
        assert_eq!(directives[1].line, 4);

        assert_eq!(directives[2].raw_path, "heavy.h");
        assert!(!directives[2].is_angled);
        assert_eq!(directives[2].line, 5);
    }

    #[test]
    fn line_comment_before_include_directive_is_not_parsed() {
        let content = r#"
// #include "commented_out.h"
#include "real.h"
"#;
        let directives = parse_include_directives(content);
        assert_eq!(directives.len(), 1);
        assert_eq!(directives[0].raw_path, "real.h");
        assert_eq!(directives[0].line, 3);
    }

    #[test]
    fn empty_angle_brackets_yield_empty_raw_path() {
        // Characterizes current behaviour: `<>` parses as an angled directive
        // with an empty path (resolution of such a directive simply misses).
        let content = "#include <>\n";
        let directives = parse_include_directives(content);
        assert_eq!(directives.len(), 1);
        assert_eq!(directives[0].raw_path, "");
        assert!(directives[0].is_angled);
        assert_eq!(directives[0].line, 1);
    }

    #[test]
    fn whitespace_variants_between_hash_and_include_are_parsed() {
        let content = "#\tinclude\t<windows.h>\n# include \"tabbed.h\"\n";
        let directives = parse_include_directives(content);
        assert_eq!(directives.len(), 2);
        assert_eq!(directives[0].raw_path, "windows.h");
        assert!(directives[0].is_angled);
        assert_eq!(directives[1].raw_path, "tabbed.h");
        assert!(!directives[1].is_angled);
    }

    #[test]
    fn unterminated_directive_is_skipped() {
        let content = "#include \"unterminated.h\n#include \"ok.h\"\n";
        let directives = parse_include_directives(content);
        assert_eq!(directives.len(), 1);
        assert_eq!(directives[0].raw_path, "ok.h");
    }
}
