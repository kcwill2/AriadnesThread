//! M23.5 T3-A -- cycle-classification views feeding the include-graph DTO.
//!
//! Two cycle currencies reach the payload layer: the legacy reported-path
//! enumeration (`IncludeCycleClass::from_paths`, retired in T3-E) and the
//! milestone's `CycleGroup` membership (`from_groups`, plan
//! 1790821308681-cycle-substrate-scc-m23_5 sec1.3). Both entry points in
//! `dto.rs` share ONE emission tail and differ only in which view they pass,
//! so node/edge byte-shape (order, keys, classes) is identical outside the
//! cycle semantics the two entry points exist to separate.

use ariadne_graph::{CycleGroup, IncludeGraph};
use ariadne_repo::FileId;
use std::collections::{HashMap, HashSet};

/// Node/edge cycle classification plus the grouped-node inspector keys
/// (`cycle_group_size`, `cycle_witness`). Built once per payload and queried
/// per element during emission; carries no emission order itself.
pub(crate) struct IncludeCycleClass {
    grouped_nodes: HashSet<FileId>,
    cyclic_pairs: HashSet<(FileId, FileId)>,
    group_size_by_file: HashMap<FileId, usize>,
    witness_labels_by_file: HashMap<FileId, Vec<String>>,
}

impl IncludeCycleClass {
    /// Legacy path-based classification (pre-M23.5, byte-compatible): every
    /// consecutive step of a reported cycle path and the closing
    /// `(last, first)` pair is a cyclic edge; every endpoint it names is a
    /// cyclic node. Carries no group inspector data -- the two new keys stay
    /// OMITTED for every node (D27 OMITS rule), keeping old-path payloads
    /// byte-identical.
    pub(crate) fn from_paths(cycles: &[Vec<FileId>]) -> Self {
        let mut grouped_nodes = HashSet::new();
        let mut cyclic_pairs = HashSet::new();
        for cycle in cycles {
            for step in cycle.windows(2) {
                grouped_nodes.insert(step[0]);
                grouped_nodes.insert(step[1]);
                cyclic_pairs.insert((step[0], step[1]));
            }
            if let (Some(&first), Some(&last)) = (cycle.first(), cycle.last()) {
                grouped_nodes.insert(first);
                grouped_nodes.insert(last);
                cyclic_pairs.insert((last, first));
            }
        }
        Self {
            grouped_nodes,
            cyclic_pairs,
            group_size_by_file: HashMap::new(),
            witness_labels_by_file: HashMap::new(),
        }
    }

    /// Group-membership classification (IV-2 + IV-3): a node is cyclic iff it
    /// appears in some group's `member_files`; an edge `u -> v` is cyclic iff
    /// `u` and `v` sit in ONE AND THE SAME group -- tested as equality of the
    /// group index both endpoints map to, not mere co-membership in *some*
    /// group (IV-1 makes the groups disjoint, so the index is unique). An
    /// edge from a grouped file into a different group's file, or into an
    /// ungrouped file, stays UNCLASSSED. A self-loop `u -> u` is cyclic iff
    /// `u` is grouped (index equality holds trivially).
    pub(crate) fn from_groups(groups: &[CycleGroup], graph: &IncludeGraph) -> Self {
        let group_index_by_member = group_index_by_member(groups);
        let mut cyclic_pairs = HashSet::new();
        for edge in &graph.edges {
            if let Some(destination) = edge.destination {
                if endpoints_share_group(&group_index_by_member, edge.source, destination) {
                    cyclic_pairs.insert((edge.source, destination));
                }
            }
        }
        let mut group_size_by_file = HashMap::new();
        let mut witness_labels_by_file = HashMap::new();
        for group in groups {
            let witness_labels = witness_path_labels(group, graph);
            for &member in &group.member_files {
                group_size_by_file.insert(member, group.member_files.len());
                witness_labels_by_file.insert(member, witness_labels.clone());
            }
        }
        Self {
            grouped_nodes: group_index_by_member.keys().copied().collect(),
            cyclic_pairs,
            group_size_by_file,
            witness_labels_by_file,
        }
    }

    /// IV-2: the node carries class `cycle` iff it is in some group.
    pub(crate) fn node_is_cycle(&self, file: FileId) -> bool {
        self.grouped_nodes.contains(&file)
    }

    /// IV-3: the edge carries class `cycle` iff both endpoints are in the
    /// same group (path view: the pair was a reported path step).
    pub(crate) fn edge_is_cycle(&self, source: FileId, destination: FileId) -> bool {
        self.cyclic_pairs.contains(&(source, destination))
    }

    /// Member count of the node's group; `None` (key OMITTED) when ungrouped.
    pub(crate) fn group_size(&self, file: FileId) -> Option<usize> {
        self.group_size_by_file.get(&file).copied()
    }

    /// The node's group witness path labels, in `representative_cycle` order
    /// (IV-5, never hash order); `None` (key OMITTED) when ungrouped.
    pub(crate) fn witness_labels(&self, file: FileId) -> Option<&[String]> {
        self.witness_labels_by_file.get(&file).map(Vec::as_slice)
    }
}

/// `member FileId -> index of its group`. IV-1 guarantees disjoint groups, so
/// each member lands in exactly one group and the mapping is unambiguous.
fn group_index_by_member(groups: &[CycleGroup]) -> HashMap<FileId, usize> {
    let mut index_by_member = HashMap::new();
    for (index, group) in groups.iter().enumerate() {
        for &member in &group.member_files {
            index_by_member.insert(member, index);
        }
    }
    index_by_member
}

/// True iff BOTH lookups hit and land on the SAME group index; any ungrouped
/// endpoint breaks the equality and yields false.
fn endpoints_share_group(
    index_by_member: &HashMap<FileId, usize>,
    source: FileId,
    destination: FileId,
) -> bool {
    match (index_by_member.get(&source), index_by_member.get(&destination)) {
        (Some(source_group), Some(destination_group)) => source_group == destination_group,
        _ => false,
    }
}

/// Path labels of a group's closed-walk witness, in `representative_cycle`
/// order (IV-5). Members the graph never registered in `file_paths` (possible
/// per the T2-B divergence case; they emit no node anyway) are omitted from
/// the label array.
fn witness_path_labels(group: &CycleGroup, graph: &IncludeGraph) -> Vec<String> {
    group
        .representative_cycle
        .iter()
        .filter_map(|member| graph.file_paths.get(member))
        .map(|path| path.to_string_lossy().to_string())
        .collect()
}
