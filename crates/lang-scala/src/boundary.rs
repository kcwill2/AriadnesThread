//! External-boundary path markers for Scala repositories (plan M20.1, IV-6).
//!
//! Single source of truth shared by two consumers:
//! [`crate::plugin::ScalaPlugin::modeling`] (locked by
//! `modeling_matches_plan_section_3_4_scala_column` and by the
//! `external_frozen_boundary_lock` in `m19_scala_discovery_test.rs`) and
//! the external-boundary checks in [`crate::resolver`]. The values must
//! stay exactly `["lib", "target", ".ivy2", ".cache"]`.

/// Repository path components that mark an external boundary: sbt/Coursier
/// artifact and build-output directories whose files the resolver must
/// never treat as in-repo dependency targets.
pub const EXTERNAL_PATH_MARKERS: &[&str] = &["lib", "target", ".ivy2", ".cache"];

/// Whether one path component names an external boundary directory.
pub(crate) fn is_external_boundary_component(component: &str) -> bool {
    EXTERNAL_PATH_MARKERS.contains(&component)
}
