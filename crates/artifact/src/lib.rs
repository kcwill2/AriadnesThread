//! Artifact-store IO: the ArtifactManager, manifest DTOs, and the bounded
//! bincode wire options that guard them (AR-26a extraction from
//! ariadne-report).

pub mod manager;
pub mod manifests;

pub use manager::*;
pub use manifests::*;
