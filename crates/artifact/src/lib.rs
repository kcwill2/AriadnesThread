//! Artifact store IO surface (AR-26a): the manifest DTOs for the staged
//! artifact set, moved out of ariadne-report; report re-exports every
//! path so all consumers resolve UNEDITED.

pub mod manifests;

pub use manifests::*;
