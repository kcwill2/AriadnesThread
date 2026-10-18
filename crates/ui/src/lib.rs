pub(crate) mod cycle_groups_view;
pub mod dto;
pub mod export;
pub mod server;
pub(crate) mod routes;
pub(crate) mod state;

pub use dto::*;
pub use export::*;
pub use server::*;
/// Wizard contract export (ui-extraction-wizard W-2b): the cli provider
/// (W-2a) implements this trait against the routes seam; the routes module
/// itself stays crate-internal.
pub use routes::api_extraction::ExtractionManifestReader;
pub use routes::api_extraction_apply::{ExtractionApplyRefusal, ExtractionPlanApplier};

pub const INDEX_HTML: &str = include_str!("../assets/index.html");
pub const THEMES_CSS: &str = include_str!("../assets/themes.css");
pub const STYLES_CSS: &str = include_str!("../assets/styles.css");
pub const API_JS: &str = include_str!("../assets/api.js");
pub const GRAPH_JS: &str = include_str!("../assets/graph.js");
pub const FILE_VIEW_JS: &str = include_str!("../assets/file-view.js");
pub const HOTSPOTS_JS: &str = include_str!("../assets/hotspots.js");
pub const TRAVERSAL_JS: &str = include_str!("../assets/traversal.js");
pub const PREVIEW_JS: &str = include_str!("../assets/preview.js");
pub const PROVENANCE_JS: &str = include_str!("../assets/provenance.js");
pub const MAIN_JS: &str = include_str!("../assets/main.js");
pub const EXTRACTION_WIZARD_JS: &str = include_str!("../assets/extraction-wizard.js");
pub const THEME_JS: &str = include_str!("../assets/theme.js");
pub const ICON_SVG: &str = include_str!("../assets/icon.svg");

/// Splash version corner token: the bundled asset stays build-invariant and
/// every real page (served or exported) carries the crate version instead.
pub const VERSION_PLACEHOLDER: &str = "__ARIADNE_VERSION__";

/// The index page with the splash build-version corner injected.
pub fn rendered_index_html() -> String {
    INDEX_HTML.replace(VERSION_PLACEHOLDER, env!("CARGO_PKG_VERSION"))
}
