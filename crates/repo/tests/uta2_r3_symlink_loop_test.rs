//! UTA2-R3 (UTB-8): directory loops must not turn repository discovery into
//! an unbounded walk. `discover_repository_files_with_progress` pushed every
//! subdirectory onto its stack with no visited set, so a junction/symlink
//! back into an ancestor re-descended the same real directory forever (audit
//! UTB-8: "a directory loop is an INFINITE WALK (hang)").
//!
//! Loop construction follows the T7a deterministic-injection precedent: the
//! failure/loop is created through the platform's own primitive - `cmd /c
//! mklink /J` (junction, no elevation needed) on Windows,
//! `std::os::unix::fs::symlink_dir` on unix. When the environment forbids
//! creation the test skips gracefully (loud eprintln, recorded in
//! tasks/uta2-r3-journal.md).
//!
//! Safety contract of this test itself: the walk runs on a spawned thread and
//! is judged through a channel `recv_timeout` bound, so a missing guard makes
//! the test FAIL (timeout, or a duplicate-record assertion once the OS path
//! ceiling truncates the walk) - it can never hang the test process.

use ariadne_repo::discover_repository_files;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;
use tempfile::TempDir;

/// Upper bound granted to a correct walk on this two-file tree. The correct
/// walk returns in milliseconds; the bound exists to catch the unguarded one.
const WALK_BOUND: Duration = Duration::from_secs(15);

/// Creates a directory-type loop `link -> target` (windows: junction,
/// unix: symlink dir). Err means the environment forbade creation.
#[cfg(windows)]
fn try_create_dir_loop(link: &Path, target: &Path) -> std::io::Result<()> {
    let outcome = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(link)
        .arg(target)
        .output()?;
    if outcome.status.success() {
        return Ok(());
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("mklink /J exited with status {:?}", outcome.status),
    ))
}

#[cfg(unix)]
fn try_create_dir_loop(link: &Path, target: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink_dir(target, link)
}

#[cfg(not(any(windows, unix)))]
fn try_create_dir_loop(_link: &Path, _target: &Path) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "no directory-loop primitive for this platform",
    ))
}

/// Runs the walk on a spawned thread behind a channel timeout, so the
/// unguarded (pre-fix) walk cannot hang this test process. Returns the
/// records, or panics with the watchdog verdict on the bound.
fn walk_within_bound(root: &Path) -> Vec<ariadne_repo::FileRecord> {
    let root_buf: PathBuf = root.to_path_buf();
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        // Receiver may already be gone if the bound tripped; ignore.
        let _result = sender.send(discover_repository_files(&root_buf));
    });
    match receiver.recv_timeout(WALK_BOUND) {
        Ok(Ok(files)) => files,
        Ok(Err(walk_error)) => panic!(
            "a looped tree must still discover Ok (the loop is skipped, \
             not an error), got: {walk_error:?}"
        ),
        Err(_timeout) => panic!(
            "UTB-8: discovery did not terminate within {:?} on a \
             directory loop - no visited-set guard is in place",
            WALK_BOUND
        ),
    }
}

#[test]
fn discovery_terminates_on_directory_loop_and_records_loop_target_once() {
    let dir = TempDir::new().expect("temp repo root");
    let root = dir.path();
    let sub = root.join("sub");
    fs::create_dir_all(&sub).expect("loop host dir");
    fs::write(sub.join("inner.cpp"), b"int inner() {}").expect("looped source");

    // The loop: `sub/loop` names `sub` itself. Pre-fix the walk re-descends
    // sub forever (until the OS path ceiling truncates it, duplicating the
    // record at every surviving level); post-fix the canonical visited set
    // skips the alias and sub is walked exactly once.
    let loop_link = sub.join("loop");
    if let Err(create_error) = try_create_dir_loop(&loop_link, &sub) {
        eprintln!(
            "SKIPPING discovery_terminates_on_directory_loop_and_records_loop_target_once: \
             environment forbids directory-loop creation ({create_error}); \
             see the skip note in tasks/uta2-r3-journal.md"
        );
        return;
    }

    let files = walk_within_bound(root);
    let recorded_paths: Vec<String> = files
        .iter()
        .map(|record| record.path.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        recorded_paths,
        vec!["sub/inner.cpp".to_string()],
        "the loop target dir must be walked exactly once and the alias \
         must neither duplicate nor re-descend it"
    );
}
