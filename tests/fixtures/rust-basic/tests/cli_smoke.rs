// rust-language R-4 fixture: the test-side file cargo compiles separately
// from the crate's library code. Discovery records it; the shaper EXCLUDES
// it from unit shaping (the `tests` exclusion tag).

#[test]
fn cli_smoke_reads_the_fixture_root() {
    let _ = rust_basic_crate_root_present();
}

fn rust_basic_crate_root_present() -> bool {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).exists()
}
