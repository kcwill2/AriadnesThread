//! External-boundary path markers for Rust repositories (rust-language R-3).
//!
//! Single source of truth for the modeling literal (shape sec. 3.3 row
//! `external_path_markers`), mirroring the `crates/lang-scala` `boundary.rs`
//! idiom. The values must stay exactly `["target", "vendor", ".cargo"]`:
//! `target` is cargo build output, `vendor` the C++-convention vendored
//! dependency directory, `.cargo` the repo-local cargo config directory. All
//! three are discovery-skipped or rarely present; the list is declared for
//! boundary-data honesty. The R-6 resolver consumes this const for its
//! external-boundary checks.

/// Repository path components that mark an external boundary: build-output
/// and dependency directories whose files resolution must never treat as
/// in-repo dependency targets.
pub const EXTERNAL_PATH_MARKERS: &[&str] = &["target", "vendor", ".cargo"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_path_markers_match_the_shape_literal() {
        assert_eq!(
            EXTERNAL_PATH_MARKERS,
            ["target", "vendor", ".cargo"],
            "shape sec. 3.3: single-sourced boundary markers, drift here would \
             split the modeling literal from the future resolver's checks"
        );
    }
}
