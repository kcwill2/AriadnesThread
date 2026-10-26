// rust-language R-4 fixture: the external/frozen side of the boundary.
// `target/` cannot be committed (and discovery traversal skips it), so the
// external marker rides `vendor/` per shape sec. 4. Discovery records this
// file as ThirdParty/Frozen; it never becomes a unit of context (no `src`
// component).

pub fn legacy() -> u32 {
    0
}
