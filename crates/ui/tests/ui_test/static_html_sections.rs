use crate::common;

#[test]
fn test_inspector_link_sections_are_collapsible() {
    let html = ariadne_ui::INDEX_HTML;

    for (list_id, title_id, count_id) in [
        ("inspUpstreamList", "inspUpstreamTitle", "inspUpstreamCount"),
        (
            "inspDownstreamList",
            "inspDownstreamTitle",
            "inspDownstreamCount",
        ),
    ] {
        let list = html
            .find(&format!("id=\"{list_id}\""))
            .unwrap_or_else(|| panic!("missing {list_id} container"));
        let open = html[..list]
            .rfind("<details")
            .unwrap_or_else(|| panic!("{list_id} must be wrapped in a <details> element"));
        let wrapper = &html[open..list];
        let tag_end = wrapper.find('>').expect("unterminated <details> tag");
        assert!(
            wrapper[..tag_end].contains("open"),
            "{list_id} section must default to expanded (open attribute)"
        );
        assert!(
            !wrapper.contains("</details>"),
            "{list_id} must sit directly inside its enclosing <details>"
        );
        let summary = wrapper
            .find("<summary")
            .unwrap_or_else(|| panic!("{list_id} section needs a <summary> before the list"));
        let summary_end = wrapper[summary..]
            .find("</summary>")
            .unwrap_or_else(|| panic!("{list_id} summary must close before the list"));
        let summary_body = &wrapper[summary..summary + summary_end];
        assert!(
            summary_body.contains(&format!("id=\"{title_id}\""))
                && summary_body.contains(&format!("id=\"{count_id}\"")),
            "{list_id} summary must keep the title ({title_id}) and count ({count_id}) visible when collapsed"
        );
        assert!(
            html[list..].contains("</details>"),
            "{list_id} section wrapper must close after the list"
        );
    }
}

// E3 lands preview.js as a NEW split asset. Read it straight from disk so this
// test compiles before the lib.rs constant exists: the red run must fail on
// missing markers/registration, never on a compile error.
fn preview_asset_text() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/preview.js");
    std::fs::read_to_string(path).unwrap_or_default()
}

#[test]
fn test_extraction_impact_preview_overlay() {
    let preview = preview_asset_text();
    let page = common::page_content() + preview.as_str();

    // Public overlay API (E-I6: a new asset, not more graph.js).
    for marker in ["window.ariadnePreview", "applyPreview", "clearPreview"] {
        assert!(
            page.contains(marker),
            "preview overlay asset must expose {marker} (E-I6)"
        );
    }

    // Node-marking classes: applied in preview.js, styled in styles.css.
    for class in ["preview-eliminated", "preview-kept", "preview-still-reachable"] {
        assert!(
            page.contains(class),
            "preview overlay must apply the {class} class"
        );
        assert!(
            ariadne_ui::STYLES_CSS.contains(&format!(".{class}")),
            "styles.css must define a rule for .{class}"
        );
    }

    // Legend replaces the heat legend with the after-state summary.
    assert!(
        page.contains("After extraction:"),
        "preview legend must carry the after-extraction summary line"
    );

    // E-I4 proof of the standalone path: the asset reads the capped sample
    // fields and admits the sample honestly with a "+N more" note.
    assert!(
        preview.contains("dropped_sample") && preview.contains("more eliminated"),
        "preview.js must drive the standalone capped-sample path (E-I4)"
    );

    // Row control lives in the api.js justification renderer.
    assert!(
        ariadne_ui::API_JS.contains("Preview after extraction"),
        "justification rows with impact need the Preview after extraction control"
    );
    assert!(
        ariadne_ui::API_JS.contains("just-preview-btn"),
        "the preview control must carry the just-preview-btn class"
    );

    // Proposal-card control rides the shared card builder (hotspots.js) and
    // names the currently selected consumer as the preview root.
    assert!(
        ariadne_ui::HOTSPOTS_JS.contains("Preview drop from selected file"),
        "proposal cards need the selection-aware preview control"
    );

    // graph.js gains only the clear-on-rerender hook (E-I6).
    assert!(
        ariadne_ui::GRAPH_JS.contains("ariadnePreview.clearPreview()"),
        "graph.js must clear the preview when the selection re-renders"
    );
    assert!(
        ariadne_ui::GRAPH_JS.lines().count() <= 960,
        "graph.js must stay within the E-I6 line budget (cap 960)"
    );

    // Help text describes the overlay in the heat map legend region.
    assert!(
        ariadne_ui::INDEX_HTML.contains("Preview after extraction"),
        "heat map help text must mention the Preview after extraction overlay"
    );
}
