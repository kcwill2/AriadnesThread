use crate::cycle_groups_view;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Lifecycle of one boot step row, serialized lowercase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProgressStepStatus {
    Pending,
    Active,
    Done,
}

/// One `[k/N]` boot step as the terminal would show it: its own 0-100 bar
/// plus the in-flight detail. `label` carries the phase text with the
/// `[k/N] ` prefix stripped (the browser composes the label from
/// `stage`/`total_stages`). `message` is empty while `pending`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgressStepView {
    pub stage: usize,
    pub label: String,
    pub percent: usize,
    pub message: String,
    pub status: ProgressStepStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerProgress {
    pub state: String,
    pub phase: String,
    pub percent: usize,
    pub message: String,
    /// Per-step rows mirroring the terminal's committed + live lines.
    /// Empty only pre-seed (standalone exports, legacy fixtures).
    #[serde(default)]
    pub steps: Vec<ProgressStepView>,
    #[serde(default)]
    pub total_stages: usize,
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
pub struct TraversalQuery {
    pub root_file_id: String,
    pub dep_file_id: String,
    /// Optional specific symbol IDs to analyze; if empty, analyzes all symbols
    /// the root uses from this dependency (from justification data).
    #[serde(default)]
    pub symbol_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraversalSubtreeNode {
    pub id: String,
    pub label: String,
    pub path: String,
    pub kind: String,
    pub is_necessary: bool,
    pub is_collateral: bool,
    pub is_decoy: bool,
    pub symbol_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraversalSubtreeEdge {
    pub source: String,
    pub target: String,
    pub is_necessary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraversalSubtree {
    pub nodes: Vec<TraversalSubtreeNode>,
    pub edges: Vec<TraversalSubtreeEdge>,
    pub total_symbols: usize,
    pub file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraversalResponse {
    pub root_file_id: String,
    pub dep_file_id: String,
    pub analyzed_symbol_ids: Vec<String>,
    pub necessary_subtree: TraversalSubtree,
    pub actual_subtree: TraversalSubtree,
    pub collateral_delta: TraversalSubtree,
    pub summary: TraversalSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraversalSummary {
    pub necessary_files: usize,
    pub actual_files: usize,
    pub collateral_files: usize,
    pub necessary_symbols: usize,
    pub actual_symbols: usize,
    pub collateral_symbols: usize,
    pub decoy_files: usize,
    pub decoy_symbols: usize,
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

/// Cap for destination-file rows per proposal (mirrors the header cap).
pub const EXTRACTION_PROPOSAL_MAX_TARGET_FILES: usize = 8;

/// One suggested destination file and the symbols it should contain.
/// `path` is repo-relative, lossy-rendered like the plan's PathBuf (plain
/// display data only — the cluster plan type never crosses into ui).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposedTargetFileDto {
    /// Repo-relative destination path suggested by the plan.
    pub path: String,
    /// Symbols destined for this file, sorted by symbol_id, capped at
    /// `EXTRACTION_PROPOSAL_MAX_NAMES`.
    pub symbol_names: Vec<String>,
    /// Uncapped member count for this file (truncation of `symbol_names`
    /// is `symbol_count - symbol_names.len()`).
    pub symbol_count: usize,
}

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
    /// Suggested destination files with per-file symbol manifests, grouped
    /// from the plan's proposed extractions; `serde(default)` keeps payloads
    /// written before this field decodable (empty = no file plan known).
    #[serde(default)]
    pub target_files: Vec<ProposedTargetFileDto>,
    /// True iff destination-file rows were capped at
    /// `EXTRACTION_PROPOSAL_MAX_TARGET_FILES` by the producer.
    #[serde(default)]
    pub target_files_truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExportPayload {
    pub repo_name: String,
    pub include_graph: Vec<CytoscapeElement>,
    pub symbol_graph: Vec<CytoscapeElement>,
    pub call_graph: Vec<CytoscapeElement>,
    pub stats: SummaryStats,
    #[serde(
        default,
        skip_serializing_if = "std::collections::HashMap::is_empty",
        serialize_with = "serialize_sorted_nested_maps"
    )]
    pub file_collateral:
        std::collections::HashMap<String, std::collections::HashMap<String, usize>>,
    #[serde(
        default,
        skip_serializing_if = "std::collections::HashMap::is_empty",
        serialize_with = "serialize_sorted_nested_maps"
    )]
    pub file_justification: std::collections::HashMap<
        String,
        std::collections::HashMap<String, JustificationEntry>,
    >,
    /// Conservative extraction proposals (top 12 by collateral delta, I3/I8);
    /// absent from the JSON when empty, so older payloads round-trip (I2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extraction_proposals: Vec<ExtractionProposalDto>,
}

/// Serialize a map with keys in ascending order: std `HashMap` iteration
/// order comes from a per-process seed, so every map in the export payload
/// must sort at emission for the `--export` bytes to be stable across
/// processes. Deserialization keeps the plain impl — JSON object key order
/// carries no semantics.
fn serialize_sorted_map<K, V, S>(
    map: &std::collections::HashMap<K, V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    K: Ord + Serialize,
    V: Serialize,
    S: serde::Serializer,
{
    let mut entries: Vec<(&K, &V)> = map.iter().collect();
    entries.sort_by_key(|(key, _)| *key);
    serializer.collect_map(entries)
}

/// Ascending-key view of an inner payload map (see `serialize_sorted_map`).
struct SortedInnerMap<'a, V>(&'a std::collections::HashMap<String, V>);

impl<V: Serialize> Serialize for SortedInnerMap<'_, V> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_sorted_map(self.0, serializer)
    }
}

/// `serialize_with` helper for the payload's nested map fields: both the
/// outer and the inner key level serialize in ascending order.
fn serialize_sorted_nested_maps<V, S>(
    map: &std::collections::HashMap<String, std::collections::HashMap<String, V>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    V: Serialize,
    S: serde::Serializer,
{
    let sorted: std::collections::BTreeMap<&String, SortedInnerMap<'_, V>> = map
        .iter()
        .map(|(key, inner)| (key, SortedInnerMap(inner)))
        .collect();
    sorted.serialize(serializer)
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
    build_file_justification_with_memo(file_metrics, include_graph, model, path_to_id, None)
}

/// C1 memo-seeding twin: `Some(memo)` builds the analyzer over an ALREADY
/// WARM shared `Arc<LoadMemo>` (the cli boot path seeds closures from the
/// pipeline stage output, so payload assembly and every later live route
/// share one warm memo); `None` keeps the cold per-build analyzer, the
/// behavior of the plain wrapper above.
///
/// A6: the per-file loop runs in parallel over `file_metrics` (rayon
/// `par_iter`, the established repo pattern of cluster's planner and the
/// pipeline's `collect_file_metrics`). Determinism is structural, not
/// incidental: the payload is keyed by numeric file ids, every symbol /
/// header / impact row inside one root group is canonically sorted by the
/// graph and dto contracts, the per-file `impact_slots` budget is local to
/// the unit, and the outer map's key order was never payload-meaningful.
/// The only shared state is the `Sync` analyzer + `Arc<LoadMemo>` (OnceLock
/// maps + Mutex guards held only for get/insert, never across derived work);
/// no inner loop here spawns nested rayon work. Locked byte-for-byte by the
/// golden/determinism tests in `tests/ui_test/justification_golden.rs`.
pub fn build_file_justification_with_memo(
    file_metrics: &[ariadne_graph::LoadMetric],
    include_graph: &ariadne_graph::IncludeGraph,
    model: &ariadne_graph::ReducedSemanticModel,
    path_to_id: &std::collections::HashMap<std::path::PathBuf, u64>,
    memo: Option<&std::sync::Arc<ariadne_graph::LoadMemo>>,
) -> std::collections::HashMap<
    String,
    std::collections::HashMap<String, JustificationEntry>,
> {
    let analyzer = match memo {
        Some(memo) => ariadne_graph::LoadAnalyzer::with_memo(
            include_graph,
            model,
            std::sync::Arc::clone(memo),
        ),
        None => ariadne_graph::LoadAnalyzer::new(include_graph, model),
    };
    let roots: Vec<(String, std::collections::HashMap<String, JustificationEntry>)> =
        file_metrics
            .par_iter()
            .filter_map(|metric| justification_root(metric, &analyzer, model, path_to_id))
            .collect();
    roots.into_iter().collect()
}

/// A6 rayon unit of work for one root file: the identical pre-A6 per-metric
/// body (rank sources, build capped entries, spend the per-file impact-slot
/// budget). `None` reproduces the serial loop's three `continue`s: target
/// unregistered, sides empty after the E-I9 header filter, or no kept
/// entries.
fn justification_root(
    metric: &ariadne_graph::LoadMetric,
    analyzer: &ariadne_graph::LoadAnalyzer,
    model: &ariadne_graph::ReducedSemanticModel,
    path_to_id: &std::collections::HashMap<std::path::PathBuf, u64>,
) -> Option<(String, std::collections::HashMap<String, JustificationEntry>)> {
    let root_id = metric
        .target_file
        .as_ref()
        .and_then(|p| path_to_id.get(p))?;
    let sides = collect_source_sides(metric);
    if sides.is_empty() {
        return None;
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
                    analyzer,
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
                analyzer,
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
    if per_root.is_empty() {
        None
    } else {
        Some((root_id.to_string(), per_root))
    }
}

#[derive(Default)]
struct SourceSides<'a> {
    justified: Option<&'a ariadne_graph::ContributingSource>,
    collateral: Option<&'a ariadne_graph::ContributingSource>,
}

/// Groups a metric's contributing sources by file id, both sides; shared by
/// the bounded payload builder and the unbounded route builder. Mirrors the
/// E-I9 needs-set rule: only includable headers (`.h`/`.hpp`/`.hxx`) are kept
/// on either side, so implementation-file rows (including the inspected root
/// itself) never pollute the justification list. Collateral-only header rows
/// survive — they ARE the unnecessary-include signal.
fn collect_source_sides(
    metric: &ariadne_graph::LoadMetric,
) -> std::collections::HashMap<u64, SourceSides<'_>> {
    let mut sides: std::collections::HashMap<u64, SourceSides> =
        std::collections::HashMap::new();
    for src in &metric.semantic_sources {
        if src.symbol_count > 0 && ariadne_graph::is_includable_header(&src.path) {
            sides.entry(src.file_id.0).or_default().justified = Some(src);
        }
    }
    for src in &metric.collateral_sources {
        if src.symbol_count > 0 && ariadne_graph::is_includable_header(&src.path) {
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

/// File nodes plus include edges classified from `CycleGroup` membership
/// (M23.5, plan sec1.3): the ONE include-element entry point in `crates/ui`
/// (the path-based one was retired in T3-E by E1), keeping the exact
/// 5-argument D27 shape. The third parameter was `&[Vec<FileId>]` (reported
/// DFS path enumeration) before M23.5 and is now `&[CycleGroup]` (the SCC
/// partition), so the earlier type must not be "restored" here.
///
/// File nodes plus include edges. D27: `file_languages`/`file_unit_kinds`
/// (numeric FileId → display string) are injected by the cli payload builder
/// from discovery records and plugin modeling; a missing entry OMITS the
/// `language`/`unit_kind` key (locked contract, no fabricated default).
///
/// - a node carries class `cycle` iff it is in some group's `member_files`
///   (IV-2);
/// - an edge carries `cycle` iff BOTH endpoints are in the SAME group
///   (IV-3): an intra-group chord the old DFS enumeration never walked as a
///   back-edge can no longer render unstyled between two `cycle`-classed
///   nodes.
/// Grouped nodes additionally carry `"cycle_group_size"` (member count) and
/// `"cycle_witness"` (the group's representative-cycle path labels in
/// `representative_cycle` order, IV-5); both follow the D27 OMITS rule -- a
/// node in no group gets NEITHER key, never `null` and never `0`.
pub fn build_include_elements(
    graph: &ariadne_graph::IncludeGraph,
    closures: &std::collections::HashMap<ariadne_repo::FileId, ariadne_graph::IncludeClosure>,
    cycle_groups: &[ariadne_graph::CycleGroup],
    file_languages: &std::collections::HashMap<u64, String>,
    file_unit_kinds: &std::collections::HashMap<u64, String>,
) -> Vec<CytoscapeElement> {
    emit_include_elements(
        graph,
        closures,
        &cycle_groups_view::IncludeCycleClass::from_groups(cycle_groups, graph),
        file_languages,
        file_unit_kinds,
    )
}

/// Shared emission tail behind the one include-element entry point: exactly
/// one node loop in canonical `(path asc, file_id asc)` order followed by one
/// edge loop in `graph.edges` order. The cycle-classification view is this
/// layer's only cycle-classification input; degree tallying, sorting,
/// kind derivation, D27 decoration and the OMITS rule for the group keys are
/// shared, so there is no duplicated emission loop (M23.5 T3-A).
fn emit_include_elements(
    graph: &ariadne_graph::IncludeGraph,
    closures: &std::collections::HashMap<ariadne_repo::FileId, ariadne_graph::IncludeClosure>,
    cycle_class: &cycle_groups_view::IncludeCycleClass,
    file_languages: &std::collections::HashMap<u64, String>,
    file_unit_kinds: &std::collections::HashMap<u64, String>,
) -> Vec<CytoscapeElement> {
    let mut elements = Vec::new();

    let mut in_degrees = std::collections::HashMap::new();
    let mut out_degrees = std::collections::HashMap::new();
    for edge in &graph.edges {
        if let Some(dest) = edge.destination {
            *out_degrees.entry(edge.source).or_insert(0) += 1;
            *in_degrees.entry(dest).or_insert(0) += 1;
        }
    }

    // Canonical (path asc, file_id asc) emission: `file_paths` is a std
    // HashMap whose seed is per-process, so raw iteration would shuffle the
    // node array on every run — `--export` must be byte-stable across
    // processes.
    let mut ordered_files: Vec<(&ariadne_repo::FileId, &std::path::PathBuf)> =
        graph.file_paths.iter().collect();
    ordered_files.sort_by_key(|(file_id, path)| (*path, **file_id));

    for (file_id, path) in ordered_files {
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
        let is_cycle = cycle_class.node_is_cycle(*file_id);
        let in_degree = in_degrees.get(file_id).copied().unwrap_or(0);
        let out_degree = out_degrees.get(file_id).copied().unwrap_or(0);

        let mut data = serde_json::json!({
            "id": file_id.0.to_string(),
            "label": fname,
            "path": path_str,
            "kind": kind,
            "closure": closure_size,
            "is_cycle": is_cycle,
            "in_degree": in_degree,
            "out_degree": out_degree,
        });
        if let Some(language) = file_languages.get(&file_id.0) {
            data["language"] = serde_json::Value::String(language.clone());
        }
        if let Some(unit_kind) = file_unit_kinds.get(&file_id.0) {
            data["unit_kind"] = serde_json::Value::String(unit_kind.clone());
        }
        if let Some(group_size) = cycle_class.group_size(*file_id) {
            data["cycle_group_size"] = serde_json::json!(group_size);
        }
        if let Some(witness_labels) = cycle_class.witness_labels(*file_id) {
            data["cycle_witness"] = serde_json::json!(witness_labels);
        }
        elements.push(CytoscapeElement {
            data,
            classes: if is_cycle { Some("cycle".into()) } else { None },
        });
    }

    for edge in &graph.edges {
        if let Some(dest) = edge.destination {
            let is_cycle = cycle_class.edge_is_cycle(edge.source, dest);
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

/// Resolves declaring-file ids to the injected path strings, order-preserving,
/// duplicates collapsed after resolution; unresolvable ids drop out (D27).
fn resolve_declaring_paths(
    ids: &[ariadne_repo::FileId],
    file_paths: &std::collections::HashMap<u64, String>,
) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut paths = Vec::new();
    for id in ids {
        if let Some(path) = file_paths.get(&id.0) {
            if seen.insert(path.clone()) {
                paths.push(path.clone());
            }
        }
    }
    paths
}

/// Symbol nodes plus symbol edges. D27 defect fix (§3.6.1): `defining_file`
/// and `path` carry the RESOLVED repo-relative path from the cli-injected
/// `file_paths` map (was the numeric FileId string), `path` falls back to the
/// empty string the source-drawer JS guard expects; `declaring_files` is the
/// additive declared-in path set. Unresolvable ids ⇒ null/empty, never panic.
pub fn build_symbol_elements(
    model: &ariadne_graph::ReducedSemanticModel,
    file_paths: &std::collections::HashMap<u64, String>,
) -> Vec<CytoscapeElement> {
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
        let defining_path = sym
            .defining_file
            .and_then(|f| file_paths.get(&f.0))
            .cloned();
        let declaring_paths = resolve_declaring_paths(&sym.declaring_files, file_paths);
        elements.push(CytoscapeElement {
            data: serde_json::json!({
                "id": sym.id.0.clone(),
                "label": sym.name.clone(),
                "kind": kind_str,
                "defining_file": defining_path.clone(),
                "declaring_files": declaring_paths,
                "path": defining_path.unwrap_or_default(),
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
