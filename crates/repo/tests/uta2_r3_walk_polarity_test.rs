//! UTA2-R3 item 4: characterization pins (NO unification) for the walk's
//! two contradictory error polarities in `discover_repository_files_with_progress`:
//!
//! - directory-open failures (`match fs::read_dir ... Err(_) => continue`,
//!   discovery.rs): SILENTLY SKIPPED - the walk continues and the overall
//!   result stays Ok.
//! - per-entry/per-file failures (`entry?`, `fs::metadata?`, and the
//!   `fs::read` inside `compute_file_hash`): PROPAGATE - one unreadable file
//!   aborts the WHOLE walk, even beside readable ones.
//!
//! The locked-file injection follows the T7a `share_mode(0)` deterministic
//! precedent (Windows). It trips the propagation polarity through the
//! hashing read; the `entry?`/`metadata?` sites are the same `?`-propagation
//! polarity but have no deterministic injection without a deletion race
//! (metadata/open succeed on a share-locked file) - the journal records that
//! adjacency argument plus the unification policy proposal (not applied).

use ariadne_repo::discover_repository_files;
use tempfile::TempDir;

/// Silent polarity pin: a failing `read_dir` at the root is swallowed by the
/// `Err(_) => continue` arm, so discovery returns Ok with zero records -
/// the "unreadable repo looks empty" class already on the deferred-findings
/// ledger as the upstream empty-Ok root.
#[test]
fn pin_read_dir_failure_polarity_walk_continues_to_empty_ok() {
    let missing_root = TempDir::new()
        .expect("temp base")
        .path()
        .join("uta2-r3-missing-root");
    let files = discover_repository_files(&missing_root).expect(
        "PINNED POLARITY: a failing root read_dir is swallowed (continue), \
         never an Err - the walk yields an empty Ok",
    );
    assert!(
        files.is_empty(),
        "a swallowed root yields zero records, got {files:?}"
    );
}

#[cfg(windows)]
mod locked_file_polarity {
    use super::*;
    use std::fs;
    use std::fs::OpenOptions;
    use std::os::windows::fs::OpenOptionsExt;

    /// Loud polarity pin: with one share-locked analyzable file beside a
    /// readable sibling, discovery must Err for the WHOLE walk - the per-
    /// file `?` sites do not skip-and-continue. This is the same propagation
    /// arm reached by `entry?`/`metadata?` failures (characterized together).
    #[test]
    fn pin_locked_analyzable_file_aborts_the_whole_walk() {
        let dir = TempDir::new().expect("temp repo root");
        let root = dir.path();
        fs::write(root.join("readable.h"), b"#pragma once\n").expect("readable sibling");
        let locked = root.join("locked.cpp");
        fs::write(&locked, b"int locked() {}").expect("locked analyzable source");
        let _exclusive_handle = OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&locked)
            .expect("exclusive (unshareable) open");

        let outcome = discover_repository_files(root);
        let err = outcome.err().expect(
            "PINNED POLARITY: a per-file read failure propagates; the walk \
             does not continue past it even with readable siblings",
        );
        let chain = format!("{err:?}");
        assert!(
            chain.contains("Failed to read file for hashing")
                && chain.contains("locked.cpp"),
            "the abort must surface the hashing read context AND name the \
             offending file, got: {chain}"
        );
    }
}
