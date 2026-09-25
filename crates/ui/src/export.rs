use crate::dto::GraphExportPayload;
use crate::INDEX_HTML;

pub fn export_standalone_html(
    payload: &GraphExportPayload,
    template: &str,
    output_path: &std::path::Path,
) -> anyhow::Result<()> {
    let tpl = if template.is_empty() {
        INDEX_HTML
    } else {
        template
    };
    let json_data = serde_json::to_string(payload)?;
    let script = format!("<script>window.__ARIADNE_DATA__ = {};</script>", json_data);
    let output = tpl.replace("/*__ARIADNE_DATA__*/", &script);
    std::fs::write(output_path, output)?;
    Ok(())
}
