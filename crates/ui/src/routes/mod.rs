//! AR-13: route-family modules extracted verbatim from server.rs.
//! `pub(crate)` keeps the crate's public surface on `server.rs` alone.

pub(crate) mod api_extraction;
pub(crate) mod api_extraction_apply;
pub(crate) mod api_graph;
pub(crate) mod api_payloads;
pub(crate) mod api_source;
