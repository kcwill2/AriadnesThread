//! R-9b3 W2 - the discovery-vs-rewrite fidelity pair (AMENDMENT-3 sec. 1d),
//! authored first (gate 2 RED: `left: false / right: true` for the honest
//! class) and split out of [`crate::dependency_rewriter`] by the hand's
//! CONDITIONAL hygiene escape: the inline growth would have passed the
//! 995-line budget (declared in `tasks/rust-language-r9b-journal.md` sec. 12).

use crate::dependency_rewriter::RustDependencyRewriter;
use ariadne_lang::DependencyRewriter;
use std::path::{Path, PathBuf};

/// R-9b3 W2 (AMENDMENT-3 sec. 1d): the Rust analog of C++'s X1(a)
/// fidelity pair (lang-cpp :347-372). Over the wired grammar (FLAT id,
/// mod.rs parent): a consumer is discovered (some probe hits via
/// find_dependency_targets) EXACTLY when the hook-built spec's rewrite
/// changes its text. The glob asymmetry (discovery hits under a
/// wildcard base while a symbol-level old dep never narrows that glob)
/// is R-8 landed law (dependency_rewriter :643-659, :523-534) - cited,
/// not re-proved: the glob class here is the aggregate-narrowing one
/// (base == honest spelling, both sides true); a glob above the base is
/// the excluded asymmetric class.
#[test]
fn discovery_and_rewrite_hit_iff_over_the_honest_spellings() {
    let moved = ["crate::models_marker".to_string()];
    let probes =
        RustDependencyRewriter.consumer_discovery_targets(Path::new("src/models/mod.rs"), &moved);
    let built = RustDependencyRewriter
        .consumer_rewrite_spec(
            &[PathBuf::from("src/models/mod.rs")],
            &[PathBuf::from("src/models/models_marker.rs")],
            &moved,
        )
        .expect("the wired payload derives a spec");
    let discovered = |text: &str| {
        probes
            .iter()
            .any(|probe| !RustDependencyRewriter.find_dependency_targets(text, probe).is_empty())
    };
    let rewritten = |text: &str| RustDependencyRewriter.rewrite(text, &built) != text;
    let honest_leaf = "use crate::models::models_marker;\nfn user() {}\n";
    for (class, text) in [
        ("honest M::X leaf", honest_leaf),
        ("colliding flat name in another module", "use crate::elsewhere::models_marker;\n"),
        ("foreign root", "use other::place::models_marker;\n"),
        (
            "commented/quoted spelling",
            "// use crate::models::models_marker;\nconst DOC: &str = \"use crate::models::models_marker;\";\n",
        ),
        ("glob at the honest base (aggregate-narrowing class)", "use crate::models::models_marker::*;\n"),
    ] {
        assert_eq!(discovered(text), rewritten(text), "fidelity pair broken for class {class}");
    }
    assert!(
        discovered(honest_leaf) && rewritten(honest_leaf),
        "the honest leaf is the POSITIVE class: discovered AND rewritten (CW-1)"
    );
}
