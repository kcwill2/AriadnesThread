//! M18.11 — FINAL C++-plugin parity suite (plan §5 "M18 acceptance",
//! §9 M18.11 first item). This is the PERMANENT lock that replaces every
//! transitional `#[deprecated]` shim and legacy hardcoded path deleted by
//! M18.11-B:
//!
//! * `resolver_parity` — `CppDependencyResolver::resolve_edges` /
//!   `resolve_target` reproduce the legacy `#include` behavior on `basic` +
//!   `expensive`, against vendored directive captures and a vendored copy of
//!   the pre-M18 LIFO closure walk (`git show ceac55f:crates/scheduler/src/
//!   tu_store.rs`'s `compute_tu_closure`, driven through the lang-cpp
//!   `IncludeResolver` which remains public and non-deprecated).
//! * `worker_backend_parity` — `PluginWorkerBackend` (body-only tasks)
//!   reproduces the deleted transitional in-process backend byte-for-byte;
//!   its oracle is a local copy of the `analyze_task` body-only branch from
//!   `git show ceac55f:crates/scheduler/src/worker.rs`, calling
//!   `ariadne_lang_cpp::CppSemanticAnalyzer::analyze_source`.
//! * `isolation_output_parity` — `IsolationEngine` + `CppSourceExtractor` +
//!   `CppDependencyRewriter` reproduce the generated-file bytes (placeholder
//!   bodies included), the `MigrationPreview` JSON, and the `format_diff_with`
//!   text captured at ceac55f from the pre-deletion code; the same bytes are
//!   cross-checked against the production transactional write path.
//! * `provenance_lock` — reduced-model `defining_file` / `declaring_files`
//!   through the public registry pipeline equal projections recomputed
//!   INDEPENDENTLY in-test (the ceac55f decl-location rule AND the older
//!   first-wins/Class-upgrade owning-file rule), never by calling the new
//!   rule's code.
//!
//! Deliberate rule for this file tree: it references NOTHING deprecated or
//! legacy-path (those symbols die in M18.11-B). Vendored oracles may call
//! `ariadne_lang_cpp::{IncludeResolver, CppSemanticAnalyzer,
//! parse_include_directives}` — all public non-deprecated lang-cpp items
//! that survive the deletions.

mod common;
mod m18_cpp_plugin_parity;
