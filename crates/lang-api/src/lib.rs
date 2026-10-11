//! Neutral plugin vocabulary shared by core pipeline crates and language
//! plugins (plan `multi-language-support-implementation_plan.md` §3.2).
//!
//! This crate depends only on `ariadne-repo` and `serde`: core crates
//! (`index`, `graph`, ...) and plugin crates (`lang-cpp`, `lang-scala`, ...)
//! both depend inward on it, which breaks the former `lang -> index` cycle
//! without duplicating any value type.

pub mod capability;
pub mod catalog;
pub mod error;
pub mod identity;
pub mod modeling;
pub mod types;
pub mod unit;
