//! Artifact generation and load reporting (AR-26a: manifest DTOs live in
//! ariadne-artifact; paths preserved).

pub mod manager;
pub use manager::*;

pub use ariadne_artifact::manifests;
pub use ariadne_artifact::manifests::*;
// AR-26a ruling rider (board_12d48349): this pub(crate) re-export keeps the
// widened helper OUT of report's flat public surface (the fn must be pub in
// ariadne-artifact until 26a2 moves its three IO consumers same-crate). The
// binding is referenced only as the shadowing mechanism, so both lints are
// intrinsic, not incidental. Revert the definition to pub(crate) and DROP
// this line in 26a2; terminal flat surface = the 12 DTOs.
#[allow(hidden_glob_reexports, unused_imports)]
pub(crate) use ariadne_artifact::manifests::collect_stage_languages;
