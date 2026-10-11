pub mod dto;
pub mod export;
pub mod server;

pub use dto::*;
pub use export::*;
pub use server::*;

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
pub const THEME_JS: &str = include_str!("../assets/theme.js");
pub const ICON_SVG: &str = include_str!("../assets/icon.svg");
