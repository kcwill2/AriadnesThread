//! UTA2-R5 row 3: `probe_compiler_status` Available-arm pins.
//!
//! The existing Missing-arm lock (`probe_of_absent_binary_is_missing_and_
//! graceful` in the plugin.rs test module) stays byte-untouched; these pins
//! land the two arms it cannot reach: a real version banner and the honest
//! empty-stdout fallback. The probe's documented contract (plugin.rs:72-76)
//! is spawn-success based - the process EXIT STATUS is never inspected - so
//! every binary below that the OS could run lands `Available`, whichever
//! stdout shape decides the payload.

use super::*;

/// One-shot extraction of what a banner means to the probe: the first
/// trimmed line that carries text.
fn first_nonempty_trimmed_line(text: &str) -> Option<&str> {
    text.lines().map(str::trim).find(|line| !line.is_empty())
}

#[test]
fn available_arm_reports_the_first_nonempty_line_of_a_real_banner() {
    // cmd.exe does not consume `--version`: it prints its version banner to
    // stdout and exits 0. The probe must surface EXACTLY the first non-empty
    // line, trimmed - independently recomputed from the process itself, not
    // from a machine-dependent literal.
    let direct = std::process::Command::new("cmd.exe")
        .arg("--version")
        .output()
        .expect("cmd.exe must spawn");
    let banner = String::from_utf8_lossy(&direct.stdout);
    let expected = first_nonempty_trimmed_line(&banner).expect("cmd.exe prints a version banner");
    assert!(
        !expected.is_empty() && !expected.contains('\n'),
        "fixture sanity: the expected payload is a single trimmed line, got {expected:?}"
    );

    match probe_compiler_status("cmd.exe") {
        ToolchainStatus::Available(version) => {
            assert_eq!(
                version, expected,
                "Available must carry the first non-empty banner line"
            );
            assert!(!version.contains('\n'), "exactly one line may be carried");
        }
        other => panic!("cmd.exe runs successfully: Available expected, got {other:?}"),
    }
}

#[test]
fn a_runnable_binary_with_empty_stdout_falls_back_to_the_binary_string() {
    // findstr --version exits 2 and writes its complaint to STDERR only
    // (empty stdout, verified on this host). Status is never inspected, so
    // this is the `Ok(out)` arm with an empty banner: the fallback must say
    // Available(<binary string>) - honest, never Missing, never a fabricated
    // version.
    assert_eq!(
        probe_compiler_status("findstr.exe"),
        ToolchainStatus::Available("findstr.exe".to_string()),
        "empty-stdout success must fall back to the binary name"
    );
}

#[test]
fn probing_the_running_test_binary_lands_available_via_the_empty_stdout_fallback() {
    // The handoff expected this specimen to yield a libtest `--version`
    // banner. It does not on this toolchain (rustc 1.99.0): libtest rejects
    // the option with "error: Unrecognized option: 'version'" on stderr and
    // panics (exit 101) while stdout stays EMPTY. Spawn still succeeds and
    // the exit status is not part of the contract, so the test binary hits
    // the Available arm through the same fallback row as findstr - pinned
    // exactly. If a future libtest implements `--version` this goes red
    // where the owner can re-pin; the behavioral invariant (never Missing,
    // never a panic) holds either way.
    let exe = std::env::current_exe().expect("the running test binary path is available");
    let exe_text = exe.to_str().expect("the test binary path is valid UTF-8");
    assert_eq!(
        probe_compiler_status(exe_text),
        ToolchainStatus::Available(exe_text.to_string()),
        "empty stdout from the spawned test binary must fall back to the binary string"
    );
}
