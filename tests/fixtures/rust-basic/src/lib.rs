// rust-language R-4 fixture: the crate root of the frozen `rust-basic`
// tree. Declares the two module forms the shaper must see: `mod models;`
// resolved by the src/models/mod.rs sibling and `mod greeting;` resolved
// by src/greeting.rs.

mod greeting;
mod models;

pub fn fixture_marker() -> &'static str {
    "rust-basic"
}
