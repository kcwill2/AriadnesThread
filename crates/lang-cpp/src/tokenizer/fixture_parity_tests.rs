//! AR-31 fixture token-stream characterization suite (P8 tokenizer rework).
//!
//! Pre-change oracle: for EVERY C++ fixture TU under `tests/fixtures/`, the
//! full [`tokenize_raw`] stream - kind, text, line, 1-based column, and the
//! `start`/`end` char-span fields - is locked byte-for-byte against a
//! committed oracle file under `crates/lang-cpp/tests/tokenizer_oracles/`.
//! These are precisely the fields the M27 underflow class perturbs and the
//! fields the M23 verbatim locator (`symbol_locator`) slices declaration
//! text by. The suite lands GREEN against the pre-rework tokenizer; the
//! rework must pass it UNEDITED (the contract's core discipline).
//!
//! Deviation D1 (journal): the handoff names `crates/lang-cpp/tests/` for
//! the suite. The oracle DATA lives there (`tokenizer_oracles/`), but the
//! test module lives here because `tokenize*`/`RawToken` are `pub(crate)`
//! behind the private `mod tokenizer` (`lib.rs` is outside the touch-set);
//! the three existing tokenizer lock modules sit in this same directory.
//!
//! Authored pure ASCII (AGENTS.md UTF-8 rule): oracle token text fields are
//! written through [`ascii_escape_field`], so multi-byte fixture characters
//! (the m27-nonascii-underflow specimens) enter the committed oracles as
//! `\xHH` escapes, never as raw bytes.

use std::fs;
use std::path::{Path, PathBuf};

use super::{tokenize_raw, TokenKind};

/// C-family source extensions the fixture walk treats as tokenizer TUs.
const TU_EXTENSIONS: [&str; 7] = ["cpp", "h", "hpp", "cc", "cxx", "hh", "hxx"];

/// Repo root, from the crate manifest dir (`crates/lang-cpp` -> two up).
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|crates_dir| crates_dir.parent())
        .expect("lang-cpp must live under <repo>/crates")
        .to_path_buf()
}

/// Appends every fixture TU (slash-separated path relative to
/// `tests/fixtures`, canonical absolute path) under `dir`, depth-first in
/// sorted order. Sorting keeps the walk - and so the oracle pairing -
/// platform-deterministic.
fn walk_tus(dir: &Path, relative_prefix: &str, sink: &mut Vec<(String, PathBuf)>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("fixture dir {dir:?} must be readable: {err}"))
        .map(|entry| entry.expect("fixture entry must be readable").path())
        .collect();
    entries.sort();
    for path in entries {
        let name = path
            .file_name()
            .expect("entry must have a name")
            .to_string_lossy()
            .replace('\\', "/");
        if path.is_dir() {
            walk_tus(&path, &format!("{relative_prefix}{name}/"), sink);
        } else if path
            .extension()
            .map(|ext| TU_EXTENSIONS.contains(&ext.to_string_lossy().to_lowercase().as_str()))
            .unwrap_or(false)
        {
            sink.push((format!("{relative_prefix}{name}"), path.clone()));
        }
    }
}

/// Every C++ fixture TU, sorted by slash-separated relative path.
fn fixture_tus() -> Vec<(String, PathBuf)> {
    let mut tus = Vec::new();
    walk_tus(&repo_root().join("tests").join("fixtures"), "", &mut tus);
    tus
}

/// Committed oracle file for one fixture TU.
fn oracle_path_for(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("tokenizer_oracles")
        .join(format!("{relative}.txt"))
}

/// Renders a token text field as pure printable ASCII: `\\` stays a
/// literal double backslash, other printable ASCII bytes pass through,
/// everything else (non-ASCII, control) becomes `\\xHH`.
fn ascii_escape_field(text: &str) -> String {
    let mut escaped = String::new();
    for byte in text.bytes() {
        match byte {
            b'\\' => escaped.push_str("\\\\"),
            0x20..=0x7e => escaped.push(byte as char),
            other => escaped.push_str(&format!("\\x{other:02X}")),
        }
    }
    escaped
}

/// Renders the full raw token stream, one newline-terminated line per
/// token: `W|text|line|column|start|end` (words) or the same with `P`
/// (puncts). This shape carries all five parity fields at once.
fn render_token_stream(source: &str) -> String {
    let mut rendered = String::new();
    for token in tokenize_raw(source) {
        let (kind, text) = match &token.kind {
            TokenKind::Word(text) => ('W', text),
            TokenKind::Punct(text) => ('P', text),
        };
        rendered.push_str(&format!(
            "{kind}|{}|{}|{}|{}|{}\n",
            ascii_escape_field(text),
            token.line,
            token.column,
            token.start,
            token.end
        ));
    }
    rendered
}

/// The load-bearing lock: every fixture TU's token stream must equal its
/// committed oracle byte-for-byte. Must pass UNEDITED before AND after the
/// tokenizer rework.
#[test]
fn tokenizer_fixture_token_streams_match_oracles() {
    let tus = fixture_tus();
    assert!(
        tus.len() >= 15,
        "fixture walk must find the committed C++ fixture TUs, found {}",
        tus.len()
    );
    let mut drifting: Vec<String> = Vec::new();
    for (relative, path) in &tus {
        let source = fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("fixture TU {relative} must read as UTF-8: {err}"));
        let rendered = render_token_stream(&source);
        let oracle_path = oracle_path_for(relative);
        let expected = fs::read_to_string(&oracle_path).unwrap_or_else(|err| {
            panic!("oracle {oracle_path:?} missing for fixture TU {relative}: {err}")
        });
        if rendered != expected {
            let drift_detail = match rendered
                .lines()
                .zip(expected.lines())
                .position(|(got, want)| got != want)
            {
                Some(index) => format!("token line {}", index + 1),
                None => format!(
                    "length change ({} rendered vs {} expected lines)",
                    rendered.lines().count(),
                    expected.lines().count()
                ),
            };
            drifting.push(format!("{relative} (first drift at {drift_detail})"));
        }
    }
    assert!(
        drifting.is_empty(),
        "token stream drift on {} fixture TU(s):\n{}",
        drifting.len(),
        drifting.join("\n")
    );
}
