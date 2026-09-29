//! Shared golden-artifact helpers for the M17 regression baseline
//! (plan `multi-language-support-implementation_plan.md` §6.1).
//!
//! Goldens live at `tests/golden/<fixture>/<artifact_name>.json` and capture
//! the JSON that production code itself writes under `artifacts/**`. Before
//! comparison (or update) every artifact JSON passes through
//! [`normalize_json`], whose rules are:
//!
//! 1. **Object key order** — the workspace uses plain `serde_json` (no
//!    `preserve_order` feature), so `Value::Object` is BTreeMap-backed and
//!    keys serialize in sorted order automatically.
//! 2. **Array element order** — treated as a multiset: every array is
//!    re-sorted by the canonical JSON serialization of its already-normalized
//!    elements (the chosen stable sort key). This absorbs HashMap/HashSet
//!    iteration-order leaks (manifest summaries, load-report symbol sets).
//!    Trade-off reviewers must know: a regression that only *reorders* an
//!    array whose producer iterates an unordered container cannot be caught
//!    by these goldens; added/removed/changed elements are still caught.
//! 3. **Absolute paths** — any occurrence of a configured root (the repo
//!    working dir, the fixture roots, or a runtime temp repo-copy root added
//!    via [`normalize_json_with_roots`]) in raw, verbatim (`\\?\`), or
//!    slash-separated form is replaced with `<ROOT>`; every remaining `\`
//!    becomes `/` so goldens are platform-neutral.
//! 4. **Timestamps** — keys named `timestamp` / `generated_at` are removed.
//!    (None of the seven baseline stages emit them today; the rule guards
//!    future fields.)
//! 5. **Machine-bound metadata** — `input_fingerprint` (SHA-256 over the
//!    absolute repo root) and `analysis_version` (deliberately bumped in M18)
//!    are masked to `<INPUT_FINGERPRINT>` / `<ANALYSIS_VERSION>`.

use anyhow::Context;
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

const ROOT_PLACEHOLDER: &str = "<ROOT>";
const FINGERPRINT_PLACEHOLDER: &str = "<INPUT_FINGERPRINT>";
const VERSION_PLACEHOLDER: &str = "<ANALYSIS_VERSION>";
const TIMESTAMP_KEYS: [&str; 2] = ["timestamp", "generated_at"];

/// Repository working directory (the workspace root that owns tests/).
pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|tests_dir| tests_dir.parent())
        .expect("integration tests must live under <repo>/tests/integration")
        .to_path_buf()
}

/// Absolute path of `tests/fixtures/<name>`.
pub fn fixture_path(name: &str) -> PathBuf {
    repo_root().join("tests").join("fixtures").join(name)
}

fn golden_dir(fixture: &str) -> PathBuf {
    repo_root().join("tests").join("golden").join(fixture)
}

fn golden_path(fixture: &str, artifact_name: &str) -> PathBuf {
    golden_dir(fixture).join(format!("{}.json", artifact_name))
}

/// True when `UPDATE_GOLDENS=1` requests golden regeneration.
pub fn update_goldens_requested() -> bool {
    matches!(std::env::var("UPDATE_GOLDENS").as_deref(), Ok(v) if v == "1")
}

/// Normalizes an artifact JSON against the default roots (repo working dir +
/// both C++ fixture roots). Idempotent; see the module docs for the rules.
pub fn normalize_json(value: Value) -> Value {
    normalize_json_with_roots(value, &[])
}

/// [`normalize_json`] plus extra roots (e.g. the temp repo-copy the pipeline
/// actually ran on) replaced by `<ROOT>`. Canonicalized forms of every root
/// are included so Windows verbatim paths are covered.
pub fn normalize_json_with_roots(value: Value, extra_roots: &[PathBuf]) -> Value {
    let mut roots = default_roots();
    roots.extend(extra_roots.iter().cloned());
    let forms = replacement_forms(&roots);
    normalize_value(value, &forms)
}

fn normalize_value(value: Value, roots: &[String]) -> Value {
    match value {
        Value::Object(map) => normalize_object(map, roots),
        Value::Array(items) => normalize_array(items, roots),
        Value::String(text) => Value::String(normalize_string(&text, roots)),
        other => other,
    }
}

fn normalize_object(map: Map<String, Value>, roots: &[String]) -> Value {
    let mut out = Map::new();
    for (key, raw) in map {
        if TIMESTAMP_KEYS.contains(&key.as_str()) {
            continue;
        }
        let value = match key.as_str() {
            "input_fingerprint" => Value::String(FINGERPRINT_PLACEHOLDER.to_string()),
            "analysis_version" => Value::String(VERSION_PLACEHOLDER.to_string()),
            _ => normalize_value(raw, roots),
        };
        out.insert(key, value);
    }
    Value::Object(out)
}

fn normalize_array(items: Vec<Value>, roots: &[String]) -> Value {
    let mut normalized: Vec<Value> = items
        .into_iter()
        .map(|item| normalize_value(item, roots))
        .collect();
    normalized.sort_by_key(|item| serde_json::to_string(item).unwrap_or_default());
    Value::Array(normalized)
}

fn normalize_string(text: &str, roots: &[String]) -> String {
    let mut out = text.to_string();
    for root in roots {
        if !root.is_empty() {
            out = out.replace(root.as_str(), ROOT_PLACEHOLDER);
        }
    }
    out.replace('\\', "/")
}

fn path_forms(root: &Path) -> Vec<String> {
    let mut forms = Vec::new();
    let mut candidates = vec![root.to_path_buf()];
    if let Ok(canonical) = root.canonicalize() {
        candidates.push(canonical);
    }
    for candidate in candidates {
        let raw = candidate.to_string_lossy().to_string();
        let stripped = raw.trim_start_matches("\\\\?\\").to_string();
        let slashed = stripped.replace('\\', "/");
        for form in [raw, stripped, slashed] {
            if !form.is_empty() && !forms.contains(&form) {
                forms.push(form);
            }
        }
    }
    forms
}

fn default_roots() -> Vec<PathBuf> {
    vec![repo_root(), fixture_path("basic"), fixture_path("expensive")]
}

fn replacement_forms(roots: &[PathBuf]) -> Vec<String> {
    let mut forms: Vec<String> = roots.iter().flat_map(|r| path_forms(r)).collect();
    // Longest first so the most specific form (verbatim `\\?\...`) is
    // replaced before the shorter forms that are its substring.
    forms.sort_by(|a, b| b.len().cmp(&a.len()));
    forms.dedup();
    forms
}

/// Compares the (already normalized, see [`normalize_json`]) artifact against
/// `tests/golden/<fixture>/<artifact_name>.json`. With `UPDATE_GOLDENS=1` the
/// file is written instead. On mismatch a readable line-diff summary is
/// printed and the test fails.
pub fn assert_golden_matches(fixture: &str, artifact_name: &str, actual_json: &Value) {
    let normalized = normalize_json(actual_json.clone());
    let actual = canonical_rendering(&normalized);
    let path = golden_path(fixture, artifact_name);

    if update_goldens_requested() {
        write_golden(&path, &actual);
        return;
    }

    let expected = fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "golden missing for fixture '{}' artifact '{}' at {:?} — generate with UPDATE_GOLDENS=1",
            fixture,
            artifact_name,
            path
        )
    });
    if expected == actual {
        return;
    }
    let summary = diff_summary(&expected, &actual);
    panic!(
        "golden mismatch for fixture '{}' artifact '{}' ({:?})\n{}",
        fixture, artifact_name, path, summary
    );
}

fn canonical_rendering(normalized: &Value) -> String {
    let mut text = serde_json::to_string_pretty(normalized).expect("normalized JSON is valid");
    text.push('\n');
    text
}

fn write_golden(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().expect("golden path has a parent"))
        .with_context(|| format!("failed to create golden dir for {:?}", path))
        .expect("golden dir creatable");
    fs::write(path, content)
        .with_context(|| format!("failed to write golden {:?}", path))
        .expect("golden writable");
}

fn diff_summary(expected: &str, actual: &str) -> String {
    const MAX_SHOWN: usize = 15;
    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();
    let mut out = format!(
        "golden lines: {}, actual lines: {}\n",
        expected_lines.len(),
        actual_lines.len()
    );
    let shared = expected_lines.len().min(actual_lines.len());
    let mut shown = 0usize;
    for i in 0..shared {
        if expected_lines[i] != actual_lines[i] {
            shown += 1;
            if shown > MAX_SHOWN {
                out.push_str("  ... (more differing lines suppressed)\n");
                break;
            }
            out.push_str(&format!(
                "line {}:\n  golden: {}\n  actual: {}\n",
                i + 1,
                expected_lines[i],
                actual_lines[i]
            ));
        }
    }
    if expected_lines.len() != actual_lines.len() {
        let (longer, tag) = if expected_lines.len() > shared {
            (&expected_lines[shared..], "golden-only")
        } else {
            (&actual_lines[shared..], "actual-only")
        };
        for (offset, line) in longer.iter().take(5).enumerate() {
            out.push_str(&format!("extra {} line {}: {}\n", tag, shared + offset + 1, line));
        }
    }
    out
}
