//! Neutral file-lookup surface for directive resolution (plan §3.3):
//! [`FileCatalog`] indexes discovered files so a plugin's
//! `DependencyResolver::resolve_target` can map a raw directive to a
//! repository [`FileId`] without touching the filesystem.

use ariadne_repo::{FileId, FileRecord};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Outcome of mapping one raw directive to repository files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveOutcome {
    /// Directive names exactly one repository file.
    Resolved(FileId),
    /// Target is outside the repository / an external boundary (plan §3.4).
    External,
    /// Wildcard directive expanded to several files (e.g. `import pkg._`).
    Wildcard(Vec<FileId>),
    /// Resolver could not map the directive.
    Unresolved,
}

/// Read-only index over discovered [`FileRecord`]s keyed by normalized
/// relative path, basename, and module path.
///
/// All keys normalize `\` to `/` so Windows- and POSIX-style paths address
/// the same entry. On duplicate basename/module keys the first record wins,
/// which is deterministic because discovery sorts records by path.
#[derive(Debug, Clone, Default)]
pub struct FileCatalog {
    by_rel_path: HashMap<String, FileId>,
    by_canonical_path: HashMap<String, FileId>,
    by_basename: HashMap<String, FileId>,
    by_module_path: HashMap<String, FileId>,
    records: HashMap<FileId, FileRecord>,
}

impl FileCatalog {
    /// Builds the catalog from discovery records.
    pub fn from_records(records: &[FileRecord]) -> Self {
        let mut catalog = Self::default();
        for record in records {
            catalog.insert(record);
        }
        catalog
    }

    fn insert(&mut self, record: &FileRecord) {
        let rel = normalize_path(&record.path.to_string_lossy());
        self.by_rel_path.entry(rel.clone()).or_insert(record.id);
        if let Some(basename) = rel.rsplit('/').next() {
            self.by_basename
                .entry(basename.to_string())
                .or_insert(record.id);
        }
        self.by_module_path
            .entry(module_key(&rel))
            .or_insert(record.id);
        self.by_canonical_path
            .entry(canonical_key(&record.canonical_path))
            .or_insert(record.id);
        self.records
            .entry(record.id)
            .or_insert_with(|| record.clone());
    }

    /// Resolve a candidate path against the catalog the way a resolver
    /// compares candidates to discovered files: component-normalized
    /// relative-path match first, then canonical-path match (M18.5: closes
    /// the legacy resolver's second match arm so absolute include-dir
    /// candidates behave identically through the neutral trait).
    pub fn by_path(&self, candidate: &Path) -> Option<FileId> {
        let key = canonical_key(candidate);
        self.by_rel_path
            .get(&key)
            .or_else(|| self.by_canonical_path.get(&key))
            .copied()
    }

    /// The full record for a resolved id (plan M18.5: the graph layer needs
    /// ownership + path to classify a resolved outcome without touching the
    /// filesystem or naming languages).
    pub fn record(&self, file_id: FileId) -> Option<&FileRecord> {
        self.records.get(&file_id)
    }

    /// Repository-relative path of a resolved id, as discovered.
    pub fn rel_path(&self, file_id: FileId) -> Option<&Path> {
        self.records.get(&file_id).map(|r| r.path.as_path())
    }

    /// Exact lookup by repository-relative path (`/` or `\` separated).
    pub fn by_rel_path(&self, rel_path: &str) -> Option<FileId> {
        self.by_rel_path.get(&normalize_path(rel_path)).copied()
    }

    /// Lookup by file name (`util/hash.h` → the file named `hash.h`).
    pub fn by_basename(&self, basename: &str) -> Option<FileId> {
        let base = normalize_path(basename);
        let base = base.rsplit('/').next().unwrap_or(&base);
        self.by_basename.get(base).copied()
    }

    /// Lookup by module path (`com/example/models/User`,
    /// `com/example/models/User.scala`, or `com.example.models.User`).
    /// The exact unified form is tried first, then with the final segment
    /// (extension or symbol) dropped.
    pub fn by_module_path(&self, module_path: &str) -> Option<FileId> {
        let unified = unify_segments(module_path);
        if let Some(id) = self.by_module_path.get(&unified) {
            return Some(*id);
        }
        let stripped = strip_last_segment(&unified)?;
        self.by_module_path.get(stripped).copied()
    }

    /// Deterministic (sorted-by-module-key) records whose module path ENDS
    /// WITH the given dotted/slash module path (plan M20 D12):
    /// `com.example.models.User` names `src/main/scala/com/example/models/User.scala`,
    /// and `com.example/models` names every file under that package dir —
    /// the suffix form exists because Scala packages omit the
    /// `src/main/scala` prefix that [`Self::by_module_path`] exact-keys on.
    /// Matching is component-boundary aware (`models.User` never matches
    /// `xmodels.User`). Zero matches yield an empty `Vec`; an empty query
    /// also yields an empty `Vec` instead of matching everything.
    pub fn by_module_path_suffix(&self, module_path: &str) -> Vec<FileId> {
        let suffix = unify_segments(module_path);
        if suffix.is_empty() {
            return Vec::new();
        }
        let mut hits: Vec<(String, FileId)> = self
            .records
            .values()
            .filter_map(|record| {
                let rel = normalize_path(&record.path.to_string_lossy());
                let key = module_key(&rel);
                ends_with_module_suffix(&key, &suffix).then_some((key, record.id))
            })
            .collect();
        hits.sort();
        hits.into_iter().map(|(_key, id)| id).collect()
    }

    pub fn len(&self) -> usize {
        self.by_rel_path.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_rel_path.is_empty()
    }
}

/// Windows `\` vs POSIX `/` normalization: every separator becomes `/`.
fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Catalog key for any path: `.`/`..` components resolved (without touching
/// the filesystem), separators unified to `/`.
fn canonical_key(path: &Path) -> String {
    normalize_relative_path(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Resolves `.` and `..` relative components of a path without requiring the
/// file to exist (plan M18.5: neutral home for the path-collapsing that the
/// relocated directive resolvers and the generic graph builder share).
pub fn normalize_relative_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => match components.last() {
                Some(&std::path::Component::ParentDir) | None => components.push(component),
                Some(&std::path::Component::RootDir) | Some(&std::path::Component::Prefix(_)) => {
                    // Cannot resolve `..` above root, so do nothing.
                }
                Some(&std::path::Component::Normal(_)) => {
                    components.pop();
                }
                _ => {}
            },
            c => components.push(c),
        }
    }
    let mut normalized = PathBuf::new();
    for c in components {
        normalized.push(c.as_os_str());
    }
    normalized
}

/// Unifies path separators and dots into a single dotted key form.
fn unify_segments(path: &str) -> String {
    normalize_path(path)
        .replace('/', ".")
        .trim_matches('.')
        .to_string()
}

/// Drops the final dotted segment (the file extension for stored paths).
fn strip_last_segment(unified: &str) -> Option<&str> {
    let dot = unified.rfind('.')?;
    (dot > 0).then(|| &unified[..dot])
}

/// Catalog key for a stored relative path: dotted form minus the extension.
fn module_key(rel_path: &str) -> String {
    let unified = unify_segments(rel_path);
    match strip_last_segment(&unified) {
        Some(stripped) if !stripped.is_empty() => stripped.to_string(),
        _ => unified,
    }
}

/// `key` ends with `suffix` on a component boundary: exactly the suffix, or
/// the suffix extended leftward by dotted segments (`prefix.suffix`).
fn ends_on_module_boundary(key: &str, suffix: &str) -> bool {
    key == suffix || key.ends_with(&format!(".{suffix}"))
}

/// Suffix test for [`FileCatalog::by_module_path_suffix`]: the full module
/// key carries the suffix (file-level, `...models.User` ends with
/// `models.User`), or its parent directory path does (package-dir level,
/// `...models` ends with `models`, so every file under the package hits).
fn ends_with_module_suffix(key: &str, suffix: &str) -> bool {
    ends_on_module_boundary(key, suffix)
        || strip_last_segment(key).is_some_and(|parent| ends_on_module_boundary(parent, suffix))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_repo::{FileKind, FileOwnership, OptimizationPolicy, SourceLanguage};
    use std::path::PathBuf;

    fn record(rel: &str) -> FileRecord {
        FileRecord {
            id: FileId::from_relative_path(&normalize_path(rel)),
            path: PathBuf::from(rel),
            canonical_path: PathBuf::from(rel),
            kind: FileKind::Other,
            size: 0,
            content_hash: String::new(),
            ownership: FileOwnership::Repository,
            optimization_policy: OptimizationPolicy::Optimizable,
            language: SourceLanguage::Cpp,
        }
    }

    fn sample_catalog() -> FileCatalog {
        FileCatalog::from_records(&[
            record("src/util/hash.h"),
            record("src/main.cpp"),
            record("com/example/models/User.scala"),
        ])
    }

    #[test]
    fn finds_by_rel_path_with_either_separator() {
        let catalog = sample_catalog();
        let expected = FileId::from_relative_path("src/main.cpp");
        assert_eq!(catalog.by_rel_path("src/main.cpp"), Some(expected));
        assert_eq!(catalog.by_rel_path("src\\main.cpp"), Some(expected));
        assert_eq!(catalog.by_rel_path("src//main.cpp"), None);
        assert_eq!(catalog.by_rel_path("missing.cpp"), None);
    }

    #[test]
    fn finds_by_basename() {
        let catalog = sample_catalog();
        assert_eq!(
            catalog.by_basename("hash.h"),
            Some(FileId::from_relative_path("src/util/hash.h"))
        );
        assert_eq!(
            catalog.by_basename("src/util/hash.h"),
            Some(FileId::from_relative_path("src/util/hash.h"))
        );
        assert_eq!(catalog.by_basename("nope.h"), None);
    }

    #[test]
    fn finds_by_module_path_with_dot_or_slash() {
        let catalog = sample_catalog();
        let expected = FileId::from_relative_path("com/example/models/User.scala");
        assert_eq!(
            catalog.by_module_path("com/example/models/User"),
            Some(expected)
        );
        assert_eq!(
            catalog.by_module_path("com.example.models.User"),
            Some(expected)
        );
        assert_eq!(catalog.by_module_path("com.example.models.Order"), None);
    }

    #[test]
    fn duplicate_keys_keep_first_record_deterministically() {
        let first = record("src/hash.h");
        let second = record("other/hash.h");
        let catalog = FileCatalog::from_records(&[first.clone(), second]);
        assert_eq!(catalog.by_basename("hash.h"), Some(first.id));
        assert_eq!(catalog.len(), 2);
    }

    #[test]
    fn empty_catalog() {
        let catalog = FileCatalog::from_records(&[]);
        assert!(catalog.is_empty());
        assert_eq!(catalog.by_basename("a.h"), None);
    }

    #[test]
    fn resolve_outcome_variants() {
        let file = FileId::from_relative_path("a.h");
        assert_eq!(
            ResolveOutcome::Resolved(file),
            ResolveOutcome::Resolved(file)
        );
        assert_ne!(ResolveOutcome::External, ResolveOutcome::Unresolved);
    }

    #[test]
    fn record_and_rel_path_expose_ownership_for_resolved_ids() {
        let mut record = record("third_party/lib/lib.h");
        record.ownership = ariadne_repo::FileOwnership::ThirdParty;
        let catalog = FileCatalog::from_records(&[record.clone()]);
        let id = record.id;
        assert_eq!(
            catalog.record(id).map(|r| r.ownership),
            Some(ariadne_repo::FileOwnership::ThirdParty)
        );
        assert_eq!(
            catalog.rel_path(id),
            Some(Path::new("third_party/lib/lib.h"))
        );
        assert!(catalog.record(FileId(9999)).is_none());
    }

    #[test]
    fn by_path_matches_rel_and_canonical_like_the_legacy_resolver() {
        let mut record = record("include/thing.h");
        record.canonical_path = PathBuf::from("/repo/abs/thing.h");
        record.path = PathBuf::from("include/thing.h");
        record.id = FileId::from_relative_path("include/thing.h");
        let id = record.id;
        let catalog = FileCatalog::from_records(&[record]);
        // Relative candidate, including one with `..` components to collapse.
        assert_eq!(catalog.by_path(Path::new("include/thing.h")), Some(id));
        assert_eq!(
            catalog.by_path(Path::new("src/../include/thing.h")),
            Some(id)
        );
        // Canonical candidate (absolute include-dir form).
        assert_eq!(catalog.by_path(Path::new("/repo/abs/thing.h")), Some(id));
        assert_eq!(catalog.by_path(Path::new("include/missing.h")), None);
    }

    // ── by_module_path_suffix (plan M20 D12) ──

    fn scala_tree_records() -> Vec<FileRecord> {
        vec![
            record("src/main/scala/com/example/models/Order.scala"),
            record("src/main/scala/com/example/models/User.scala"),
            record("src/main/scala/com/example/models/all.scala"),
            record("src/main/scala/com/example/services/OrderService.scala"),
        ]
    }

    fn scala_module_id(rel: &str) -> FileId {
        FileId::from_relative_path(rel)
    }

    #[test]
    fn suffix_lookup_hits_full_module_path_with_dot_or_slash() {
        let catalog = FileCatalog::from_records(&scala_tree_records());
        let order = scala_module_id("src/main/scala/com/example/models/Order.scala");
        assert_eq!(
            catalog.by_module_path_suffix("com.example.models.Order"),
            vec![order]
        );
        assert_eq!(
            catalog.by_module_path_suffix("com/example/models/Order"),
            vec![order]
        );
        assert_eq!(
            catalog.by_module_path_suffix("src/main/scala/com/example/models/Order"),
            vec![order],
            "a longer module key also ends with the given suffix"
        );
    }

    #[test]
    fn suffix_lookup_lists_every_file_under_a_package_dir() {
        let catalog = FileCatalog::from_records(&scala_tree_records());
        assert_eq!(
            catalog.by_module_path_suffix("com/example/models"),
            vec![
                scala_module_id("src/main/scala/com/example/models/Order.scala"),
                scala_module_id("src/main/scala/com/example/models/User.scala"),
                scala_module_id("src/main/scala/com/example/models/all.scala"),
            ],
            "sorted by module key (Order < User < all); the services file is not under the package"
        );
    }

    #[test]
    fn suffix_lookup_respects_component_boundaries() {
        let catalog = FileCatalog::from_records(&[record(
            "src/main/scala/com/example/xmodels/Order.scala",
        )]);
        assert!(
            catalog.by_module_path_suffix("com.example.models.Order").is_empty(),
            "`xmodels` must not match the `models` component"
        );
    }

    #[test]
    fn suffix_lookup_is_empty_for_non_match_and_empty_query() {
        let catalog = FileCatalog::from_records(&scala_tree_records());
        assert!(catalog.by_module_path_suffix("com.example.nope").is_empty());
        assert!(
            catalog.by_module_path_suffix("").is_empty(),
            "an empty query must not match every record"
        );
        assert!(FileCatalog::default()
            .by_module_path_suffix("com.example.models")
            .is_empty());
    }

    #[test]
    fn suffix_lookup_order_is_deterministic_across_constructions() {
        let ascending = FileCatalog::from_records(&scala_tree_records());
        let mut reversed_records = scala_tree_records();
        reversed_records.reverse();
        let reversed = FileCatalog::from_records(&reversed_records);
        assert_eq!(
            ascending.by_module_path_suffix("com.example.models"),
            reversed.by_module_path_suffix("com.example.models")
        );
    }

    #[test]
    fn normalize_relative_path_collapses_dots_without_touching_disk() {
        assert_eq!(
            normalize_relative_path(Path::new("a/./b/../c")),
            PathBuf::from("a/c")
        );
        assert_eq!(
            normalize_relative_path(Path::new("a/b/../../c")),
            PathBuf::from("c")
        );
        assert_eq!(
            normalize_relative_path(Path::new("../a")),
            PathBuf::from("../a")
        );
        assert_eq!(
            normalize_relative_path(Path::new("a//b///c")),
            PathBuf::from("a/b/c")
        );
    }
}
