//! Policy and ownership classification logic for repository discovery.

use crate::types::{FileOwnership, OptimizationPolicy};
use std::path::Path;

/// Classifies a file's ownership and optimization policy based on path rules and directory patterns.
pub fn classify_ownership_and_policy(
    rel_path: &Path,
    is_toolchain: bool,
) -> (FileOwnership, OptimizationPolicy) {
    if is_toolchain {
        return (FileOwnership::Toolchain, OptimizationPolicy::Frozen);
    }

    let rel_str = rel_path.to_string_lossy().replace('\\', "/");
    let lower_rel_str = rel_str.to_lowercase();

    // Check directory name components
    let components: Vec<String> = rel_path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
        .collect();

    for comp in &components {
        if comp == "third_party"
            || comp == "third-party"
            || comp == "vendor"
            || comp == "deps"
            || comp == "external"
        {
            return (FileOwnership::ThirdParty, OptimizationPolicy::Frozen);
        }
        if comp == "generated"
            || comp == "gen"
            || comp == "build"
            || comp == "out"
            || comp == "target"
            || comp == "$build"
        {
            return (FileOwnership::Generated, OptimizationPolicy::Frozen);
        }
    }

    // Explicit path pattern rules
    if lower_rel_str.starts_with("include/public/") || lower_rel_str.contains("/public/") {
        return (FileOwnership::Repository, OptimizationPolicy::Contract);
    }

    (FileOwnership::Repository, OptimizationPolicy::Optimizable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ownership_classification() {
        let (own, pol) = classify_ownership_and_policy(Path::new("src/main.cpp"), false);
        assert_eq!(own, FileOwnership::Repository);
        assert_eq!(pol, OptimizationPolicy::Optimizable);

        let (own, pol) =
            classify_ownership_and_policy(Path::new("third_party/fmt/format.h"), false);
        assert_eq!(own, FileOwnership::ThirdParty);
        assert_eq!(pol, OptimizationPolicy::Frozen);
    }

    #[test]
    fn test_generated_directory_detection() {
        let (own, pol) =
            classify_ownership_and_policy(Path::new("src/generated/parser.cpp"), false);
        assert_eq!(own, FileOwnership::Generated);
        assert_eq!(pol, OptimizationPolicy::Frozen);
    }

    #[test]
    fn test_policy_precedence() {
        let (own, pol) = classify_ownership_and_policy(Path::new("include/public/api.h"), false);
        assert_eq!(own, FileOwnership::Repository);
        assert_eq!(pol, OptimizationPolicy::Contract);
    }

    /// UTA2-R3 item 2: full keyword table. Every third-party keyword and
    /// every generated keyword, at leading and mid-path positions, maps to
    /// its class + Frozen, independent of the rest of the path.
    #[test]
    fn pin_third_party_and_generated_keyword_table() {
        let third_party_keywords = ["third_party", "third-party", "vendor", "deps", "external"];
        for keyword in third_party_keywords {
            for probe in [
                format!("{keyword}/lib.cpp"),
                format!("src/{keyword}/nested/lib.cpp"),
            ] {
                let (ownership, policy) =
                    classify_ownership_and_policy(Path::new(&probe), false);
                assert_eq!(
                    (ownership, policy),
                    (FileOwnership::ThirdParty, OptimizationPolicy::Frozen),
                    "component `{keyword}` in `{probe}` is a third-party frozen boundary"
                );
            }
        }

        let generated_keywords = ["generated", "gen", "build", "out", "target", "$build"];
        for keyword in generated_keywords {
            for probe in [
                format!("{keyword}/gen.cpp"),
                format!("src/{keyword}/nested/gen.cpp"),
            ] {
                let (ownership, policy) =
                    classify_ownership_and_policy(Path::new(&probe), false);
                assert_eq!(
                    (ownership, policy),
                    (FileOwnership::Generated, OptimizationPolicy::Frozen),
                    "component `{keyword}` in `{probe}` is generated-frozen"
                );
            }
        }
    }

    /// UTA2-R3 item 2: keyword matching is case-insensitive (components are
    /// lowercased before comparison) and component-EXACT (a component merely
    /// containing a keyword is not a hit).
    #[test]
    fn pin_keyword_matching_is_case_insensitive_and_component_exact() {
        for probe in ["Third_Party/x.h", "VENDOR/x.h", "src/Gen/x.h", "src/Build/x.h"] {
            let (ownership, policy) = classify_ownership_and_policy(Path::new(probe), false);
            assert_eq!(policy, OptimizationPolicy::Frozen, "{probe} folds to a keyword");
            let expected = if probe.contains('_') || probe.starts_with("VENDOR") {
                FileOwnership::ThirdParty
            } else {
                FileOwnership::Generated
            };
            assert_eq!(ownership, expected, "{probe} classifies via the folded keyword");
        }

        for probe in [
            "vendorish/x.h",
            "vendor_libs/x.h",
            "targeting/y.h",
            "outsized/z.h",
            "third-partydeps/w.h",
        ] {
            let (ownership, policy) = classify_ownership_and_policy(Path::new(probe), false);
            assert_eq!(
                (ownership, policy),
                (FileOwnership::Repository, OptimizationPolicy::Optimizable),
                "`{probe}` only CONTAINS a keyword; whole-component match must not fire"
            );
        }
    }

    /// UTA2-R3 item 2: the component loop returns at the FIRST matching
    /// component, so cross-class keyword order is positional, not class
    /// precedence (vendor-before-gen vs gen-before-vendor flip the class).
    #[test]
    fn pin_first_matching_component_wins_keyword_order() {
        let (ownership, policy) = classify_ownership_and_policy(Path::new("vendor/gen/x.cpp"), false);
        assert_eq!(
            (ownership, policy),
            (FileOwnership::ThirdParty, OptimizationPolicy::Frozen),
            "vendor comes first"
        );
        let (ownership, policy) = classify_ownership_and_policy(Path::new("gen/vendor/x.cpp"), false);
        assert_eq!(
            (ownership, policy),
            (FileOwnership::Generated, OptimizationPolicy::Frozen),
            "gen comes first"
        );
    }

    /// UTA2-R3 item 2: contract rule pins - the prefix arm, the mid-path
    /// contains arm, case-insensitivity via the lowered string, and the two
    /// near-miss shapes that must NOT hit (a `public` first segment without
    /// the `include/` prefix; a keyword that only prefixes `/public/`).
    #[test]
    fn pin_public_contract_rule_arms_and_near_misses() {
        for probe in ["include/public/api.h", "src/public/x.h", "INCLUDE/Public/api.h"] {
            let (ownership, policy) = classify_ownership_and_policy(Path::new(probe), false);
            assert_eq!(
                (ownership, policy),
                (FileOwnership::Repository, OptimizationPolicy::Contract),
                "`{probe}` hits a contract arm (prefix or mid-path contains)"
            );
        }
        for probe in ["public/x.h", "src/publicsneak/x.h", "include/publicx/api.h"] {
            let (ownership, policy) = classify_ownership_and_policy(Path::new(probe), false);
            assert_eq!(
                (ownership, policy),
                (FileOwnership::Repository, OptimizationPolicy::Optimizable),
                "`{probe}` misses both contract arms (no `include/public/` prefix, \
                 no `/public/` component) - quirk pinned"
            );
        }
        // The keyword loop runs BEFORE the contract patterns, so a keyword
        // dir wins even when the contract text is present later.
        let (ownership, policy) = classify_ownership_and_policy(Path::new("vendor/public/x.h"), false);
        assert_eq!(
            (ownership, policy),
            (FileOwnership::ThirdParty, OptimizationPolicy::Frozen),
            "keyword loop precedes the contract patterns"
        );
    }

    /// REPORT (UTA2-R3 item 2): `is_toolchain=true` is never passed by any
    /// production caller. The single production call site
    /// (`discovery.rs`: `classify_ownership_and_policy(rel_path, false)`)
    /// hardcodes `false`, and a repo-wide search finds no other production
    /// caller. This arm is dead in production; pinned here so its behavior
    /// is on record (flag short-circuits everything, including path rules).
    /// Removal or wiring is an owner decision, not this hand's.
    #[test]
    fn pin_toolchain_flag_arm_is_production_dead() {
        for probe in ["src/main.cpp", "vendor/x.cpp", "include/public/api.h"] {
            let (ownership, policy) = classify_ownership_and_policy(Path::new(probe), true);
            assert_eq!(
                (ownership, policy),
                (FileOwnership::Toolchain, OptimizationPolicy::Frozen),
                "the flag short-circuits before any path rule: `{probe}`"
            );
        }
    }
}
