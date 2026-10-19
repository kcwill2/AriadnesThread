//! UTA2-R5 row 4: the external-boundary marker table is a FROZEN four, and
//! the component predicate discriminates exactly - superstrings, case
//! variants, and near-miss spellings must never classify as external.
//!
//! The consumers (plugin modeling, resolver external policy) are pinned
//! elsewhere (`modeling_matches_plan_section_3_4_scala_column`, the m19
//! `external_frozen_boundary_lock`); these pins lock the source of truth
//! itself against silent table drift and predicate sloppiness.

use super::{is_external_boundary_component, EXTERNAL_PATH_MARKERS};

#[test]
fn external_path_markers_are_the_frozen_four_in_declaration_order() {
    assert_eq!(
        EXTERNAL_PATH_MARKERS,
        &["lib", "target", ".ivy2", ".cache"],
        "the marker table is frozen content AND order (boundary.rs doc contract)"
    );
    assert_eq!(EXTERNAL_PATH_MARKERS.len(), 4, "no silent fifth marker");
}

#[test]
fn every_frozen_marker_recognizes_its_exact_component() {
    for marker in EXTERNAL_PATH_MARKERS {
        assert!(
            is_external_boundary_component(marker),
            "{marker:?} must classify as an external boundary component"
        );
    }
}

#[test]
fn near_miss_components_never_classify_as_external_boundary() {
    // Each row stresses one discrimination axis of the exact-match `contains`
    // predicate: superstring (`targets` extends `target`), case (`LIB` vs
    // `lib`), prefix/suffix adjacency, missing dot, and whitespace padding.
    let near_misses = [
        "targets", // superstring of `target`
        "LIB",     // case variant of `lib`
        "Lib",     // mixed-case variant
        "TARGET",  // case variant of `target`
        ".TARGET", // case variant with dot borrowed from the dot-prefixed rows
        "cache",   // `.cache` minus its dot
        ".cache2", // `.cache` superstring
        ".ivy",    // `.ivy2` truncated
        "ivy2",    // `.ivy2` minus its dot
        "libr",    // `lib` superstring
        "targe",   // `target` truncated
        "",        // empty component
        " lib",    // leading space
        "lib ",    // trailing space
    ];
    for component in near_misses {
        assert!(
            !is_external_boundary_component(component),
            "{component:?} must NOT classify as an external boundary component"
        );
    }
}
