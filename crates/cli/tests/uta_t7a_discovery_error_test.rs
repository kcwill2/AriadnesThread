//! UTA-T7a (H7a, UTA-B7c): a repository-discovery failure must fail the
//! measure and discover-build-contexts commands loudly. The former
//! `repository.discover().ok()` swallowed the error and let the commands
//! proceed over an EMPTY file set - measure reported Ok with zero
//! translation units and discover-build-contexts persisted a fingerprint
//! computed over zero file hashes while returning Ok.
//!
//! Failure injection is Windows-specific by construction: an analyzable
//! file opened with `share_mode(0)` makes the discovery walker fail with
//! ERROR_SHARING_VIOLATION when it hashes that file. (Upstream repo
//! discovery only swallows read_dir errors, never per-file read errors,
//! so the failure is deterministic here.)

#![cfg(windows)]

use std::fs::{self, OpenOptions};
use std::os::windows::fs::OpenOptionsExt;
use std::path::PathBuf;

/// A temp repo root holding one analyzable header, opened with no sharing
/// so repository discovery cannot hash it. The returned guard owns the
/// handle: keep it alive for the whole command call.
fn repo_root_with_unreadable_source() -> (tempfile::TempDir, std::fs::File, PathBuf) {
    let dir = tempfile::tempdir().expect("temp repo root");
    let root = dir.path().to_path_buf();
    let locked_source = root.join("locked.h");
    fs::write(&locked_source, "#pragma once\n").expect("locked source is creatable");
    let exclusive_handle = OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&locked_source)
        .expect("locked source is openable without sharing");
    (dir, exclusive_handle, root)
}

#[test]
fn measure_build_performance_fails_loudly_on_discovery_failure() {
    let (_dir, _exclusive_handle, root) = repo_root_with_unreadable_source();
    let outcome = ariadne_cli::commands::measure::measure_build_performance(&root);
    let err = outcome.err().expect(
        "UTA-B7c: a repository discovery failure must fail the measure \
         command, not proceed over an empty file set",
    );
    let chain = format!("{err:?}");
    assert!(
        chain.contains("repository discovery failed"),
        "the surfaced error must name the failing discovery stage, got: {chain}"
    );
}

#[test]
fn discover_build_contexts_fails_loudly_on_discovery_failure() {
    let (_dir, _exclusive_handle, root) = repo_root_with_unreadable_source();
    let outcome = ariadne_cli::commands::discover::discover_build_contexts(&root);
    let err = outcome.err().expect(
        "UTA-B7c: a repository discovery failure must fail the command, \
         not persist a fingerprint over zero file hashes",
    );
    let chain = format!("{err:?}");
    assert!(
        chain.contains("repository discovery failed"),
        "the surfaced error must name the failing discovery stage, got: {chain}"
    );
}
