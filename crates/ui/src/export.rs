use crate::dto::GraphExportPayload;
use crate::{
    API_JS, FILE_VIEW_JS, GRAPH_JS, HOTSPOTS_JS, ICON_SVG, INDEX_HTML, MAIN_JS, PREVIEW_JS, PROVENANCE_JS,
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
    // Brand art under file:// has no routes: the favicon is meaningless and
    // the header mark must arrive as inline markup. Ids neonGradient/neonGlow
    // are unique among inline SVGs here (the app page keeps <img> precisely
    // to keep that guarantee cheap - shape D6).
    html = html.replace(
        "<link rel=\"icon\" type=\"image/svg+xml\" href=\"/icon.svg\">",
        "",
    );
    html = html.replace(
        "<img class=\"brand-mark\" src=\"/icon.svg\" alt=\"Ariadne's Thread logo\">",
        ICON_SVG.trim_end(),
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
    let script = format!(
        "window.__ARIADNE_DATA__ = {};",
        escape_json_for_script(&json_data)
    );
    let output = tpl.replace("/*__ARIADNE_DATA__*/", &script);
    std::fs::write(output_path, output)?;
    Ok(())
}

/// Script-context JSON escape (S3/S4, AR-01): the payload is embedded inside
/// a script block of the standalone page, where a raw `<` lets a hostile
/// string close the tag and U+2028/U+2029 act as line terminators inside the
/// JS object literal. Re-encoding them as JSON escape sequences is
/// parse-identical: none of the three can appear in JSON structural syntax,
/// only inside string values, so the substitution is total and safe.
fn escape_json_for_script(json: &str) -> String {
    json.replace('<', "\\u003c")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}
