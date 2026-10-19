//! Standalone-export asset-inlining lock, extracted VERBATIM from
//! `static_html.rs` (UTA2-R1 successor session): that module reached 1004
//! lines once the handoff's 9th asset-table row landed, crossing the R2
//! 1000-line ceiling. Pure relocation - the assertion bodies are untouched
//! (journal: tasks/uta2-r1-journal.md).

use crate::common;

#[test]
fn test_standalone_export_inlines_all_split_assets() {
    let temp_dir = tempfile::tempdir().unwrap();
    let out_path = temp_dir.path().join("standalone.html");
    ariadne_ui::export_standalone_html(&common::empty_export_payload(), "", &out_path).unwrap();

    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(
        content.contains("window.__ARIADNE_DATA__"),
        "export must still embed the data payload"
    );
    assert!(
        content.contains("<style>"),
        "export must inline the stylesheet: file:// has no /styles.css"
    );
    assert!(
        content.contains("async function init()"),
        "export must inline api.js"
    );
    assert!(
        content.contains("function compileMatcher"),
        "export must inline graph.js"
    );
    assert!(
        content.contains("function openSourceDrawer"),
        "export must inline file-view.js"
    );
    assert!(
        content.contains("function getBurdenMaps"),
        "export must inline hotspots.js"
    );
    assert!(
        content.contains("window.ariadnePreview"),
        "export must inline preview.js"
    );
    assert!(
        content.contains("function renderNodeProvenance("),
        "export must inline provenance.js"
    );
    assert!(
        content.contains("function triggerFirstVisitExperience"),
        "export must inline main.js"
    );
    assert!(
        !content.contains("src=\"/app/"),
        "export must not reference external JS asset routes"
    );
    assert!(
        !content.contains("href=\"/styles.css\""),
        "export must not reference the external CSS route"
    );
}
