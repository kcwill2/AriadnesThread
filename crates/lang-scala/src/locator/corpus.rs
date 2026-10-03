//! Corpus guard over the frozen m19/m21 `scala-basic` fixtures (plan M23,
//! T5): locating must re-slice every declared symbol byte-exactly, survive
//! a CRLF checkout unchanged modulo EOL, and never be corrupted by string
//! interpolation or quoted code. Read-only — the fixtures stay untouched.

use super::locate_bodies;
use crate::parser::program_of;
use std::path::{Path, PathBuf};

const CORPUS: &[&str] = &[
    "Main.scala",
    "models/User.scala",
    "models/Order.scala",
    "models/all.scala",
    "services/OrderService.scala",
];

fn read_fixture(rel_path: &str) -> String {
    let full: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/scala-basic/src/main/scala/com/example")
        .join(rel_path);
    std::fs::read_to_string(&full)
        .unwrap_or_else(|e| panic!("fixture {rel_path} must be readable: {e}"))
}

fn wrapped(namespace: &str, body: &str) -> String {
    if namespace.is_empty() {
        body.to_string()
    } else {
        format!("package {namespace}\n\n{body}")
    }
}

#[test]
fn every_fixture_symbol_locates_its_own_region_and_reparses_to_itself() {
    for rel_path in CORPUS {
        let source = read_fixture(rel_path);
        let program = program_of(&source);
        assert!(!program.decls.is_empty(), "{rel_path} must yield decls");
        for decl in &program.decls {
            let located = locate_bodies(
                &source,
                &[(decl.id.0.clone(), decl.kind)],
            );
            assert_eq!(
                located.len(),
                1,
                "{}: {} must locate",
                rel_path,
                decl.id.0
            );
            let body = &located[0].body;
            assert!(
                source.contains(body.as_str()),
                "{}: region for {} must be a verbatim substring of \
                 the source, got {:?}",
                rel_path,
                decl.id.0,
                body
            );
            let reprogram = program_of(&wrapped(&program.namespace, body));
            let names: Vec<(String, ariadne_lang::SymbolKind)> = reprogram
                .decls
                .iter()
                .map(|d| (d.id.0.clone(), d.kind))
                .collect();
            assert_eq!(
                names,
                vec![(decl.id.0.clone(), decl.kind)],
                "{rel_path}: the located region, wrapped in the file's \
                 package clause, must re-parse to exactly the one symbol it \
                 came from — interpolation/comments/strings never corrupt a \
                 region boundary"
            );
        }
    }
}

#[test]
fn order_scala_case_class_region_is_byte_locked() {
    let source = read_fixture("models/Order.scala");
    let located = locate_bodies(
        &source,
        &[("com.example.models.Order".to_string(), ariadne_lang::SymbolKind::Class)],
    );
    assert_eq!(located.len(), 1);
    let start = source
        .find("case class Order(")
        .expect("fixture shape locked by m19/m21");
    let end = source
        .rfind('}')
        .expect("fixture has a closing brace")
        + '}'.len_utf8();
    assert_eq!(
        located[0].body.as_str(),
        &source[start..end],
        "region = from `case class` through the matching `}}`, byte-equal to \
         the fixture's own region text (EOL-agnostic on any checkout)"
    );
}

#[test]
fn interpolated_splice_survives_the_slice_verbatim() {
    let source = read_fixture("services/OrderService.scala");
    let located = locate_bodies(
        &source,
        &[(
            "com.example.services.OrderService".to_string(),
            ariadne_lang::SymbolKind::Class,
        )],
    );
    assert_eq!(located.len(), 1);
    assert!(
        located[0]
            .body
            .contains("s\"${user.name} ordered item ${order.id}\""),
        "the s-interpolated line of `summarize` must ride inside the region \
         verbatim: {} ",
        located[0].body
    );
    assert!(
        located[0].body.ends_with('}'),
        "the region ends at the class's closing brace, not at the string"
    );
}

#[test]
fn crlf_variant_of_the_corpus_locates_the_same_regions() {
    for rel_path in CORPUS {
        let source = read_fixture(rel_path);
        let lf_bodies: Vec<String> = {
            let program = program_of(&source);
            program
                .decls
                .iter()
                .map(|decl| {
                    let mut located =
                        locate_bodies(&source, &[(decl.id.0.clone(), decl.kind)]);
                    assert_eq!(
                        located.len(),
                        1,
                        "{}: {} locates",
                        rel_path,
                        decl.id.0
                    );
                    located.remove(0).body
                })
                .collect()
        };
        let crlf_source = source.replace("\r\n", "\n").replace('\n', "\r\n");
        let program = program_of(&crlf_source);
        assert_eq!(program.decls.len(), lf_bodies.len(), "{rel_path}");
        for (decl, lf_body) in program.decls.iter().zip(&lf_bodies) {
            let located = locate_bodies(&crlf_source, &[(decl.id.0.clone(), decl.kind)]);
            assert_eq!(
                located.len(),
                1,
                "{}: {} locates under CRLF",
                rel_path,
                decl.id.0
            );
            assert_eq!(
                located[0].body,
                lf_body.replace("\r\n", "\n").replace('\n', "\r\n"),
                "{}: {} slices identically under CRLF",
                rel_path,
                decl.id.0
            );
        }
    }
}
