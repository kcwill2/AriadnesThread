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
