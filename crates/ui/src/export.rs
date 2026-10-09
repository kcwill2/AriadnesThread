use crate::dto::GraphExportPayload;
use crate::{
    API_JS, FILE_VIEW_JS, GRAPH_JS, HOTSPOTS_JS, INDEX_HTML, MAIN_JS, PREVIEW_JS, PROVENANCE_JS,
    STYLES_CSS, THEMES_CSS, THEME_JS, TRAVERSAL_JS,
};
use std::borrow::Cow;

/// Reassemble the split UI assets into the self-contained single-file page.
/// A standalone export is opened via `file://`, where `/styles.css`,
/// `/themes.css` and `/app/*.js` routes do not exist, so every asset must be
/// inlined — a missed stylesheet silently boots the report unthemed.
fn standalone_template() -> String {
    // Link order is themes.css (tokens) then styles.css (rules); inlining in
    // the same order preserves the token-before-rule cascade for the export.
    let mut html = INDEX_HTML.replace(
        "<link rel=\"stylesheet\" href=\"/themes.css\">",
        &format!("<style>\n{}\n</style>", THEMES_CSS.trim_end()),
    );
    html = html.replace(
        "<link rel=\"stylesheet\" href=\"/styles.css\">",
        &format!("<style>\n{}\n</style>", STYLES_CSS.trim_end()),
    );
    for (name, js) in [
        ("api.js", API_JS),
        ("graph.js", GRAPH_JS),
        ("traversal.js", TRAVERSAL_JS),
        ("file-view.js", FILE_VIEW_JS),
        ("hotspots.js", HOTSPOTS_JS),
        ("preview.js", PREVIEW_JS),
        ("provenance.js", PROVENANCE_JS),
        ("main.js", MAIN_JS),
        ("theme.js", THEME_JS),
    ] {
        let tag = format!("<script src=\"/app/{name}\"></script>");
        html = html.replace(&tag, &format!("<script>\n{}\n</script>", js.trim_end()));
    }
    html
}

pub fn export_standalone_html(
    payload: &GraphExportPayload,
    template: &str,
    output_path: &std::path::Path,
) -> anyhow::Result<()> {
    let tpl: Cow<'_, str> = if template.is_empty() {
        Cow::Owned(standalone_template())
    } else {
        Cow::Borrowed(template)
    };
    let json_data = serde_json::to_string(payload)?;
    let script = format!("window.__ARIADNE_DATA__ = {};", json_data);
    let output = tpl.replace("/*__ARIADNE_DATA__*/", &script);
    std::fs::write(output_path, output)?;
    Ok(())
}
