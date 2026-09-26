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
}
