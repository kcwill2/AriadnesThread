use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerProgress {
    pub state: String,
    pub phase: String,
    pub percent: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraverseQuery {
    pub graph_type: String,
    pub center: String,
    pub depth: Option<usize>,
    pub direction: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathQuery {
    pub graph_type: String,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub graph_type: Option<String>,
    pub q: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub graph_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntrypointsResponse {
    pub graph_type: String,
    pub cycles: Vec<Vec<String>>,
    pub top_hubs: Vec<CytoscapeElement>,
    pub roots: Vec<CytoscapeElement>,
    pub leaves: Vec<CytoscapeElement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CytoscapeElement {
    pub data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryStats {
    pub total_files: usize,
    pub total_symbols: usize,
    pub total_calls: usize,
    pub cycle_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollateralResponse {
    pub target_id: String,
    pub total_collateral: usize,
    pub direct_collateral: std::collections::HashMap<String, usize>,
    pub transitive_collateral: std::collections::HashMap<String, usize>,
}

/// Capped payload summary of one `ExtractionImpact` (E-I4): counts plus at
/// most `IMPACT_DROPPED_SAMPLE_MAX` basenames taken from the already
/// path-asc-sorted `dropped_from_root`. Closure sizes keep the graph crate's
/// seed-excluded convention (no +1 for the root).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpactSummary {
    pub dropped_from_root_count: usize,
    pub still_reachable_count: usize,
    pub dropped_sample: Vec<String>,
    pub dropped_sample_truncated: bool,
    pub root_closure_before: usize,
    pub root_closure_after: usize,
    pub symbols_no_longer_parsed: usize,
}

/// Symbol-attributed split of one include edge (root file -> source header):
/// which symbols justify the edge and which ride along as collateral payload.
/// Name lists are capped per I3; `justified_count`/`collateral_count` stay the
/// raw uncapped counts and match `file_collateral` (same LoadMetric source).
/// `impact` is the bounded extraction-impact summary (E-I4), present only on
/// eligible entries; the key is ABSENT from JSON when `None`
/// (`skip_serializing_if` — locked by the serde key tests).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JustificationEntry {
    pub justified_symbols: Vec<String>,
    pub collateral_symbols: Vec<String>,
    pub justified_count: usize,
    pub collateral_count: usize,
    pub justified_truncated: bool,
    pub collateral_truncated: bool,
    pub required_headers: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impact: Option<ImpactSummary>,
}

/// One unbounded contributor row of the live `/api/justification` route.
/// `impact` carries the FULL unbounded `ExtractionImpact` (E-I4: only the
/// payload path summarizes); the entry-level summary stays `None` here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceJustification {
    pub source_id: String,
    pub source_label: String,
    pub entry: JustificationEntry,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impact: Option<ariadne_graph::ExtractionImpact>,
}

/// Full per-file justification detail served by `/api/justification` (I3:
/// caps exist only in the export payload; this route is the unbounded twin).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileJustificationResponse {
    pub target_id: String,
    pub total_justified: usize,
    pub total_collateral: usize,
    pub entries: Vec<SourceJustification>,
}

/// Bounded payload caps (I3): contributor entries per root, symbol names per
/// side per entry, required-header basenames per entry. The live route passes
/// `usize::MAX` instead of these values to stay unbounded.
const JUSTIFICATION_MAX_SOURCES: usize = 16;
const JUSTIFICATION_MAX_NAMES: usize = 12;
const JUSTIFICATION_MAX_HEADERS: usize = 8;

/// E-I4 impact bounds: dropped-header basenames per summary sample, and how
/// many entries per root file may carry a summary (selection follows the
/// existing rank order — the top-ranked eligible sources win the slots).
const IMPACT_DROPPED_SAMPLE_MAX: usize = 8;
const IMPACT_MAX_ENTRIES_PER_ROOT: usize = 8;

/// Extraction-proposal payload caps (I3/I8): top proposals per payload,
/// symbol names per proposal (I6: sorted by SymbolId before capping; the UI
/// infers truncation from `symbol_names.len() < total_symbols`), required
/// header basenames per proposal. Name/header caps reuse the justification
/// caps.
pub const EXTRACTION_PROPOSAL_MAX: usize = 12;
pub const EXTRACTION_PROPOSAL_MAX_NAMES: usize = JUSTIFICATION_MAX_NAMES;
pub const EXTRACTION_PROPOSAL_MAX_HEADERS: usize = JUSTIFICATION_MAX_HEADERS;

/// One Conservative extraction proposal shown in both surfaces (I5/I8):
/// plain display data only — the cluster crate's `OptimizationPlan` never
/// crosses into the ui crate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractionProposalDto {
    pub plan_id: String,
    pub target_file: String,
    pub target_file_id: String,
    pub problem: String,
    pub collateral_load_before: usize,
    pub collateral_load_after: usize,
    pub symbol_names: Vec<String>,
    pub total_symbols: usize,
    pub required_headers: Vec<String>,
    pub consumer_tu_count: usize,
    pub risk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExportPayload {
    pub repo_name: String,
    pub include_graph: Vec<CytoscapeElement>,
    pub symbol_graph: Vec<CytoscapeElement>,
    pub call_graph: Vec<CytoscapeElement>,
    pub stats: SummaryStats,
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub file_collateral:
        std::collections::HashMap<String, std::collections::HashMap<String, usize>>,
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub file_justification: std::collections::HashMap<
        String,
        std::collections::HashMap<String, JustificationEntry>,
    >,
    /// Conservative extraction proposals (top 12 by collateral delta, I3/I8);
    /// absent from the JSON when empty, so older payloads round-trip (I2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extraction_proposals: Vec<ExtractionProposalDto>,
}

/// Maps `root FileId -> source FileId -> per-edge justification`, keyed like
/// `file_collateral`. Sources are ranked `(combined count desc, FileId asc)`
/// and capped (I3); symbol names are sorted by SymbolId then capped, with the
/// missing-from-model fallback to the id string.
pub fn build_file_justification(
    file_metrics: &[ariadne_graph::LoadMetric],
    include_graph: &ariadne_graph::IncludeGraph,
    model: &ariadne_graph::ReducedSemanticModel,
    path_to_id: &std::collections::HashMap<std::path::PathBuf, u64>,
) -> std::collections::HashMap<
    String,
    std::collections::HashMap<String, JustificationEntry>,
> {
    let analyzer = ariadne_graph::LoadAnalyzer::new(include_graph, model);
    let mut out: std::collections::HashMap<
        String,
        std::collections::HashMap<String, JustificationEntry>,
    > = std::collections::HashMap::new();
    for metric in file_metrics {
        let Some(root_id) = metric
            .target_file
            .as_ref()
            .and_then(|p| path_to_id.get(p))
        else {
            continue;
        };
        let sides = collect_source_sides(metric);
        if sides.is_empty() {
            continue;
        }
        let ranked = rank_source_ids(&sides);

        let mut per_root = std::collections::HashMap::new();
        let mut impact_slots = 0usize;
        for id in ranked.into_iter().take(JUSTIFICATION_MAX_SOURCES) {
            let s = &sides[&id];
            let (justified_symbols, justified_truncated) =
                justification_names(s.justified, model, JUSTIFICATION_MAX_NAMES);
            let (collateral_symbols, collateral_truncated) =
                justification_names(s.collateral, model, JUSTIFICATION_MAX_NAMES);
            let required_headers = s
                .justified
                .map(|j| {
                    justification_required_headers(
                        &analyzer,
                        &j.symbols,
                        &j.path,
                        JUSTIFICATION_MAX_HEADERS,
                    )
                })
                .unwrap_or_default();
            let justified_count = s.justified.map_or(0, |x| x.symbol_count);
            let collateral_count = s.collateral.map_or(0, |x| x.symbol_count);
            let mut impact = None;
            if impact_slots < IMPACT_MAX_ENTRIES_PER_ROOT {
                impact = entry_extraction_impact(
                    &analyzer,
                    ariadne_repo::FileId(*root_id),
                    id,
                    s.justified,
                    justified_count,
                    collateral_count,
                )
                .as_ref()
                .map(impact_summary_from);
                if impact.is_some() {
                    impact_slots += 1;
                }
            }
            per_root.insert(
                id.to_string(),
                JustificationEntry {
                    justified_symbols,
                    collateral_symbols,
                    justified_count,
                    collateral_count,
                    justified_truncated,
                    collateral_truncated,
                    required_headers,
                    impact,
                },
            );
        }
        if !per_root.is_empty() {
            out.insert(root_id.to_string(), per_root);
        }
    }
    out
}

#[derive(Default)]
struct SourceSides<'a> {
    justified: Option<&'a ariadne_graph::ContributingSource>,
    collateral: Option<&'a ariadne_graph::ContributingSource>,
}

/// Groups a metric's contributing sources by file id, both sides; shared by
/// the bounded payload builder and the unbounded route builder.
fn collect_source_sides(
    metric: &ariadne_graph::LoadMetric,
) -> std::collections::HashMap<u64, SourceSides<'_>> {
    let mut sides: std::collections::HashMap<u64, SourceSides> =
        std::collections::HashMap::new();
    for src in &metric.semantic_sources {
        if src.symbol_count > 0 {
            sides.entry(src.file_id.0).or_default().justified = Some(src);
        }
    }
    for src in &metric.collateral_sources {
        if src.symbol_count > 0 {
            sides.entry(src.file_id.0).or_default().collateral = Some(src);
        }
    }
    sides
}

/// Contributor ids ordered `(combined count desc, id asc)` per I6.
fn rank_source_ids(sides: &std::collections::HashMap<u64, SourceSides<'_>>) -> Vec<u64> {
    let combined = |id: &u64| -> usize {
        let s = &sides[id];
        s.justified.map_or(0, |x| x.symbol_count) + s.collateral.map_or(0, |x| x.symbol_count)
    };
    let mut ranked: Vec<u64> = sides.keys().copied().collect();
    ranked.sort_by(|a, b| combined(b).cmp(&combined(a)).then(a.cmp(b)));
    ranked
}

/// Unbounded justification detail for one root file, served by the live
/// `/api/justification` route: every contributing source, full symbol-name
/// lists (truncation flags stay false), per-entry required headers with the
/// source's own basename excluded. Same ranking contract as the payload (I6).
/// Eligible entries carry the FULL `ExtractionImpact` (E-I4: only the payload
/// path summarizes); no per-root cap on this route.
pub fn build_file_justification_detail(
    target_id: &str,
    root: ariadne_repo::FileId,
    metric: &ariadne_graph::LoadMetric,
    analyzer: &ariadne_graph::LoadAnalyzer,
    model: &ariadne_graph::ReducedSemanticModel,
) -> FileJustificationResponse {
    let sides = collect_source_sides(metric);
    let ranked = rank_source_ids(&sides);
    let mut entries = Vec::with_capacity(ranked.len());
    let mut total_justified = 0usize;
    let mut total_collateral = 0usize;
    for id in ranked {
        let s = &sides[&id];
        let justified_count = s.justified.map_or(0, |x| x.symbol_count);
        let collateral_count = s.collateral.map_or(0, |x| x.symbol_count);
        let source_label = s
            .justified
            .or(s.collateral)
            .map(|src| path_basename(&src.path))
            .unwrap_or_default();
        let (justified_symbols, justified_truncated) =
            justification_names(s.justified, model, usize::MAX);
        let (collateral_symbols, collateral_truncated) =
            justification_names(s.collateral, model, usize::MAX);
        let required_headers = s
            .justified
            .map(|j| justification_required_headers(analyzer, &j.symbols, &j.path, usize::MAX))
            .unwrap_or_default();
        total_justified += justified_count;
        total_collateral += collateral_count;
        let impact = entry_extraction_impact(
            analyzer,
            root,
            id,
            s.justified,
            justified_count,
            collateral_count,
        );
        entries.push(SourceJustification {
            source_id: id.to_string(),
            source_label,
            entry: JustificationEntry {
                justified_symbols,
                collateral_symbols,
                justified_count,
                collateral_count,
                justified_truncated,
                collateral_truncated,
                required_headers,
                impact: None,
            },
            impact,
        });
    }
    FileJustificationResponse {
        target_id: target_id.to_string(),
        total_justified,
        total_collateral,
        entries,
    }
}

/// E-I3/E-I4 eligibility + one `extraction_impact` call for a single kept
/// entry: `None` when the edge is not eligible (either side empty, or the
/// self-edge `root == source`) or the ids are unknown to the graph. Callers
/// summarize for the payload and embed the raw struct for the route; the BFS
/// runs at most once per kept entry.
fn entry_extraction_impact(
    analyzer: &ariadne_graph::LoadAnalyzer,
    root: ariadne_repo::FileId,
    source_id: u64,
    justified: Option<&ariadne_graph::ContributingSource>,
    justified_count: usize,
    collateral_count: usize,
) -> Option<ariadne_graph::ExtractionImpact> {
    if justified_count == 0 || collateral_count == 0 || root.0 == source_id {
        return None;
    }
    let symbols = justified?.symbols.as_slice();
    analyzer.extraction_impact(root, ariadne_repo::FileId(source_id), symbols)
}

/// Map a full `ExtractionImpact` to its bounded payload summary (E-I4): counts
/// plus the first `IMPACT_DROPPED_SAMPLE_MAX` basenames of the path-asc-sorted
/// `dropped_from_root`; the `truncated` flag is set iff that list exceeds the
/// cap. Closure sizes pass through unchanged (already seed-excluded by the
/// graph crate — never re-add the root).
pub fn impact_summary_from(impact: &ariadne_graph::ExtractionImpact) -> ImpactSummary {
    let dropped_from_root_count = impact.dropped_from_root.len();
    ImpactSummary {
        dropped_from_root_count,
        still_reachable_count: impact.still_reachable.len(),
        dropped_sample: impact
            .dropped_from_root
            .iter()
            .take(IMPACT_DROPPED_SAMPLE_MAX)
            .map(|h| path_basename(&h.path))
            .collect(),
        dropped_sample_truncated: dropped_from_root_count > IMPACT_DROPPED_SAMPLE_MAX,
        root_closure_before: impact.root_closure_before,
        root_closure_after: impact.root_closure_after,
        symbols_no_longer_parsed: impact.symbols_no_longer_parsed,
    }
}

/// Names for one side: sorted by SymbolId, capped at `max_names` (I3 payload
/// caps; the live route passes `usize::MAX`), missing symbols fall back to
/// their SymbolId string.
fn justification_names(
    side: Option<&ariadne_graph::ContributingSource>,
    model: &ariadne_graph::ReducedSemanticModel,
    max_names: usize,
) -> (Vec<String>, bool) {
    let Some(side) = side else {
        return (Vec::new(), false);
    };
    let mut ids: Vec<&ariadne_index::SymbolId> = side.symbols.iter().collect();
    ids.sort();
    let truncated = ids.len() > max_names;
    let names = ids
        .into_iter()
        .take(max_names)
        .map(|id| {
            model
                .symbols
                .get(id)
                .map(|r| r.name.clone())
                .unwrap_or_else(|| id.0.clone())
        })
        .collect();
    (names, truncated)
}

/// Basename required-header set for one edge's justified symbols: the source
/// header's own basename never appears, results are sorted and capped at
/// `max_headers` (the live route passes `usize::MAX`).
fn justification_required_headers(
    analyzer: &ariadne_graph::LoadAnalyzer,
    justified: &[ariadne_index::SymbolId],
    source_path: &std::path::Path,
    max_headers: usize,
) -> Vec<String> {
    if justified.is_empty() {
        return Vec::new();
    }
    let excluded = path_basename(source_path);
    let mut headers: Vec<String> = analyzer
        .required_headers_for_symbols(justified)
        .into_iter()
        .map(|(_, path, _)| path_basename(&path))
        .filter(|name| *name != excluded)
        .collect();
    headers.sort();
    headers.dedup();
    headers.truncate(max_headers);
    headers
}

/// Mirrors the graph crate's basename fallback for paths without a final
/// component (e.g. `..`).
fn path_basename(path: &std::path::Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

pub fn build_include_elements(
    graph: &ariadne_graph::IncludeGraph,
    closures: &std::collections::HashMap<ariadne_repo::FileId, ariadne_graph::IncludeClosure>,
    cycles: &[Vec<ariadne_repo::FileId>],
) -> Vec<CytoscapeElement> {
    let mut elements = Vec::new();
    let mut cycle_nodes = std::collections::HashSet::new();
    let mut cycle_edges = std::collections::HashSet::new();
    for cycle in cycles {
        for w in cycle.windows(2) {
            cycle_nodes.insert(w[0]);
            cycle_nodes.insert(w[1]);
            cycle_edges.insert((w[0], w[1]));
        }
        if let (Some(&first), Some(&last)) = (cycle.first(), cycle.last()) {
            cycle_nodes.insert(first);
            cycle_nodes.insert(last);
            cycle_edges.insert((last, first));
        }
    }

    let mut in_degrees = std::collections::HashMap::new();
    let mut out_degrees = std::collections::HashMap::new();
    for edge in &graph.edges {
        if let Some(dest) = edge.destination {
            *out_degrees.entry(edge.source).or_insert(0) += 1;
            *in_degrees.entry(dest).or_insert(0) += 1;
        }
    }

    for (file_id, path) in &graph.file_paths {
        let path_str = path.to_string_lossy();
        let fname = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&path_str);
        let kind = if fname.ends_with(".h") || fname.ends_with(".hpp") || fname.ends_with(".hxx") {
            "HEADER"
        } else {
            "SOURCE"
        };
        let closure_size = closures.get(file_id).map(|c| c.closure_size).unwrap_or(0);
        let is_cycle = cycle_nodes.contains(file_id);
        let in_degree = in_degrees.get(file_id).copied().unwrap_or(0);
        let out_degree = out_degrees.get(file_id).copied().unwrap_or(0);

        elements.push(CytoscapeElement {
            data: serde_json::json!({
                "id": file_id.0.to_string(),
                "label": fname,
                "path": path_str,
                "kind": kind,
                "closure": closure_size,
                "is_cycle": is_cycle,
                "in_degree": in_degree,
                "out_degree": out_degree,
            }),
            classes: if is_cycle { Some("cycle".into()) } else { None },
        });
    }

    for edge in &graph.edges {
        if let Some(dest) = edge.destination {
            let is_cycle = cycle_edges.contains(&(edge.source, dest));
            elements.push(CytoscapeElement {
                data: serde_json::json!({
                    "id": format!("{}-{}", edge.source.0, dest.0),
                    "source": edge.source.0.to_string(),
                    "target": dest.0.to_string(),
                    "kind": format!("{:?}", edge.kind),
                    "is_cycle": is_cycle,
                }),
                classes: if is_cycle { Some("cycle".into()) } else { None },
            });
        }
    }
    elements
}

pub fn build_symbol_elements(model: &ariadne_graph::ReducedSemanticModel) -> Vec<CytoscapeElement> {
    let mut elements = Vec::new();
    let mut in_degrees = std::collections::HashMap::new();
    let mut out_degrees = std::collections::HashMap::new();

    let valid_edges: Vec<_> = model
        .edges
        .iter()
        .filter(|e| {
            model.symbols.contains_key(&e.src_symbol) && model.symbols.contains_key(&e.dst_symbol)
        })
        .collect();

    for edge in &valid_edges {
        *out_degrees.entry(edge.src_symbol.clone()).or_insert(0) += 1;
        *in_degrees.entry(edge.dst_symbol.clone()).or_insert(0) += 1;
    }

    for sym in model.symbols.values() {
        let kind_str = format!("{:?}", sym.kind).to_uppercase();
        let in_degree = in_degrees.get(&sym.id).copied().unwrap_or(0);
        let out_degree = out_degrees.get(&sym.id).copied().unwrap_or(0);
        let path_str = sym
            .defining_file
            .map(|f| f.0.to_string())
            .unwrap_or_default();
        elements.push(CytoscapeElement {
            data: serde_json::json!({
                "id": sym.id.0.clone(),
                "label": sym.name.clone(),
                "kind": kind_str,
                "defining_file": sym.defining_file.map(|f| f.0.to_string()),
                "path": path_str,
                "declarations_count": sym.declarations.len(),
                "in_degree": in_degree,
                "out_degree": out_degree,
            }),
            classes: None,
        });
    }
    for edge in &valid_edges {
        let is_incomplete = edge.kind == ariadne_index::DependencyKind::Incomplete;
        elements.push(CytoscapeElement {
            data: serde_json::json!({
                "id": format!("{}-{}", edge.src_symbol.0, edge.dst_symbol.0),
                "source": edge.src_symbol.0.clone(),
                "target": edge.dst_symbol.0.clone(),
                "kind": format!("{:?}", edge.kind),
                "incomplete": is_incomplete,
            }),
            classes: if is_incomplete {
                Some("incomplete".into())
            } else {
                Some("complete".into())
            },
        });
    }
    elements
}

pub fn build_call_elements(callgraph: &ariadne_graph::CallGraph) -> Vec<CytoscapeElement> {
    let mut elements = Vec::new();
    let mut in_degrees = std::collections::HashMap::new();
    let mut out_degrees = std::collections::HashMap::new();
    for edge in &callgraph.edges {
        *out_degrees.entry(edge.caller.clone()).or_insert(0) += 1;
        *in_degrees.entry(edge.callee.clone()).or_insert(0) += 1;
    }

    for node in &callgraph.nodes {
        let in_degree = in_degrees.get(&node.symbol_id).copied().unwrap_or(0);
        let out_degree = out_degrees.get(&node.symbol_id).copied().unwrap_or(0);
        let path_str = if let Some(loc) = &node.location {
            format!("{}:{}", loc.file_id.0, loc.line)
        } else {
            node.owning_file.0.to_string()
        };
        elements.push(CytoscapeElement {
            data: serde_json::json!({
                "id": node.symbol_id.0.clone(),
                "label": node.name.clone(),
                "kind": "FUNCTION",
                "owning_file": node.owning_file.0.to_string(),
                "path": path_str,
                "line": node.location.as_ref().map(|l| l.line),
                "in_degree": in_degree,
                "out_degree": out_degree,
            }),
            classes: None,
        });
    }
    for edge in &callgraph.edges {
        let is_self_recursion = edge.caller == edge.callee;
        elements.push(CytoscapeElement {
            data: serde_json::json!({
                "id": format!("{}-{}", edge.caller.0, edge.callee.0),
                "source": edge.caller.0.clone(),
                "target": edge.callee.0.clone(),
                "call_count": edge.call_count,
                "is_cycle": is_self_recursion,
            }),
            classes: if is_self_recursion {
                Some("cycle".into())
            } else {
                None
            },
        });
    }
    elements
}

/// Per-stage persistence reuse reported by the artifact pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageInfo {
    pub stage: String,
    pub reuse: String,
}

/// Live persistence/watcher status surfaced through `/api/status`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StageStatus {
    pub stages: Vec<StageInfo>,
    pub last_refresh: Option<u64>,
    pub rebuilding: bool,
    pub reused_tus: usize,
    pub reanalyzed_tus: usize,
}
