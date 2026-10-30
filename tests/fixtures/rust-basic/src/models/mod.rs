// rust-language R-4 fixture: `mod.rs` sibling style - the directory module
// that `mod models;` in src/lib.rs resolves to. Declares the nested module.

pub mod user;

pub fn models_marker() -> &'static str {
    "models"
}
