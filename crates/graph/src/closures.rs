//! D1 (plan 1790608181551): whole-graph include-closure tables computed by
//! SCC condensation + reverse-topological dynamic programming instead of one
//! BFS per file (was O(V·(V+E)); now O(V+E) plus one union per reach edge).
//!
//! The closure convention is defined by the frozen BFS oracle in `graph.rs`
//! (`compute_closures_bfs_oracle`): `transitive_includes(X)` is the set of
//! files reachable from X via ONE OR MORE include edges, NEVER containing X
//! itself — the oracle pre-visits the seed, so inside a cycle a member's
//! closure lists its siblings but excludes itself. Condensation reproduces
//! this exactly: a member's closure is (its SCC minus itself) ∪ the union of
//! the member sets of all successor SCCs; a successor SCC can never contain
//! the seed (mutual reachability would merge it into the seed's own SCC), so
//! only the seed's own component needs the self-removal.
//!
//! `direct_includes` is untouched by this rewrite: it stays exactly the
//! sorted/dedup output of `IncludeGraph::adjacency()` (self-loops included
//! there, as the BFS version also emitted them).
//!
//! Determinism (I3): Tarjan explores nodes in ascending `FileId` order over
//! the sorted/deduped adjacency, member sets and successor lists are sorted,
//! and every closure vector is sorted before `IncludeClosure::new`, so the
//! result never depends on hash iteration order. Tarjan pops an SCC only
//! after all SCCs it reaches are already popped, so the output order IS a
//! reverse topological order of the condensation — the DP walks components
//! in that order and every successor set it unions is final by then.
//!
//! No recursion anywhere: the classic Tarjan DFS runs on an explicit frame
//! stack, so deep include chains cannot overflow the call stack.

use crate::types::IncludeClosure;
use ariadne_repo::FileId;
use std::collections::{HashMap, HashSet};

/// Computes the full `FileId -> IncludeClosure` table for the given
/// (sorted/deduped) adjacency and the file set to scope the table to: every
/// registered file plus every edge source plus every resolved destination.
pub(crate) fn compute_closures(
    adjacency: &HashMap<FileId, Vec<FileId>>,
    all_files: &HashSet<FileId>,
) -> HashMap<FileId, IncludeClosure> {
    let mut nodes: Vec<FileId> = all_files.iter().copied().collect();
    nodes.sort();

    let (components, component_of) = tarjan_components(&nodes, adjacency);
    let successors = component_successor_sets(&components, &component_of, adjacency);
    let reach = component_reach_sets(&components, &successors);

    assemble_closures(&components, &reach, adjacency)
}

/// Iterative Tarjan SCC. Returns the member sets in POP ORDER (a reverse
/// topological order of the condensation DAG) plus the node -> component
/// index map. Exploration order is ascending `FileId`, so both outputs are
/// deterministic.
fn tarjan_components(
    nodes: &[FileId],
    adjacency: &HashMap<FileId, Vec<FileId>>,
) -> (Vec<Vec<FileId>>, HashMap<FileId, usize>) {
    let mut indices: HashMap<FileId, usize> = HashMap::new();
    let mut lowlinks: HashMap<FileId, usize> = HashMap::new();
    let mut on_stack: HashSet<FileId> = HashSet::new();
    let mut tarjan_stack: Vec<FileId> = Vec::new();
    let mut components: Vec<Vec<FileId>> = Vec::new();
    let mut next_index: usize = 0;

    for &root in nodes {
        if indices.contains_key(&root) {
            continue;
        }
        indices.insert(root, next_index);
        lowlinks.insert(root, next_index);
        next_index += 1;
        tarjan_stack.push(root);
        on_stack.insert(root);

        let mut frames: Vec<(FileId, usize)> = vec![(root, 0)];
        explore_frames(
            &mut frames,
            adjacency,
            &mut indices,
            &mut lowlinks,
            &mut on_stack,
            &mut tarjan_stack,
            &mut components,
            &mut next_index,
        );
    }

    let mut component_of: HashMap<FileId, usize> = HashMap::with_capacity(nodes.len());
    for (index, members) in components.iter().enumerate() {
        for &member in members {
            component_of.insert(member, index);
        }
    }
    (components, component_of)
}

/// Runs one root's explicit-stack DFS to completion: frames carry the node
/// and the cursor into its (sorted) successor list, replacing the recursive
/// call whose stack depth the iterative rewrite exists to cap.
#[allow(clippy::too_many_arguments)]
fn explore_frames(
    frames: &mut Vec<(FileId, usize)>,
    adjacency: &HashMap<FileId, Vec<FileId>>,
    indices: &mut HashMap<FileId, usize>,
    lowlinks: &mut HashMap<FileId, usize>,
    on_stack: &mut HashSet<FileId>,
    tarjan_stack: &mut Vec<FileId>,
    components: &mut Vec<Vec<FileId>>,
    next_index: &mut usize,
) {
    while let Some(frame) = frames.last_mut() {
        let node = frame.0;
        let successors = adjacency.get(&node).map(Vec::as_slice).unwrap_or(&[]);
        let cursor = frame.1;
        if cursor < successors.len() {
            frame.1 += 1;
            let next = successors[cursor];
            match indices.get(&next) {
                Some(&child_index) => {
                    if on_stack.contains(&next) {
                        let low = lowlinks.get_mut(&node).expect("node on DFS frame has a lowlink");
                        *low = (*low).min(child_index);
                    }
                }
                None => {
                    indices.insert(next, *next_index);
                    lowlinks.insert(next, *next_index);
                    *next_index += 1;
                    tarjan_stack.push(next);
                    on_stack.insert(next);
                    frames.push((next, 0));
                }
            }
            continue;
        }

        frames.pop();
        let node_index = *indices.get(&node).expect("node on DFS frame has an index");
        let node_low = *lowlinks.get(&node).expect("node on DFS frame has a lowlink");
        if node_low == node_index {
            pop_component(node, tarjan_stack, on_stack, components);
        }
        if let Some(parent) = frames.last().map(|frame| frame.0) {
            let parent_low = lowlinks.get_mut(&parent).expect("parent on DFS frame has a lowlink");
            *parent_low = (*parent_low).min(node_low);
        }
    }
}

/// Pops the SCC rooted at `root` off the Tarjan stack (inclusive) and pushes
/// its sorted member set onto the component list.
fn pop_component(
    root: FileId,
    tarjan_stack: &mut Vec<FileId>,
    on_stack: &mut HashSet<FileId>,
    components: &mut Vec<Vec<FileId>>,
) {
    let mut members: Vec<FileId> = Vec::new();
    loop {
        let popped = tarjan_stack.pop().expect("root is on the Tarjan stack");
        on_stack.remove(&popped);
        members.push(popped);
        if popped == root {
            break;
        }
    }
    members.sort();
    components.push(members);
}

/// Distinct successor component indices per component (deduped, sorted),
/// built from the condensation edges; intra-component edges are dropped.
fn component_successor_sets(
    components: &[Vec<FileId>],
    component_of: &HashMap<FileId, usize>,
    adjacency: &HashMap<FileId, Vec<FileId>>,
) -> Vec<Vec<usize>> {
    let mut sets: Vec<HashSet<usize>> = vec![HashSet::new(); components.len()];
    for (index, members) in components.iter().enumerate() {
        for &member in members {
            if let Some(destinations) = adjacency.get(&member) {
                for &destination in destinations {
                    let other = component_of
                        .get(&destination)
                        .expect("every resolved destination is in all_files");
                    if *other != index {
                        sets[index].insert(*other);
                    }
                }
            }
        }
    }
    sets.into_iter()
        .map(|set| {
            let mut sorted: Vec<usize> = set.into_iter().collect();
            sorted.sort();
            sorted
        })
        .collect()
}

/// Reach member set per component, computed in Tarjan pop order (= reverse
/// topological): reach(C) = members(C) ∪ ⋃ reach(D) over condensation edges
/// C→D. The largest successor set is reused as the union base to keep chain
/// graphs near-linear.
fn component_reach_sets(
    components: &[Vec<FileId>],
    successors: &[Vec<usize>],
) -> Vec<HashSet<FileId>> {
    let mut reach: Vec<HashSet<FileId>> = Vec::with_capacity(components.len());
    for (index, members) in components.iter().enumerate() {
        let base = successors[index]
            .iter()
            .max_by_key(|&&successor| reach[successor].len())
            .copied();
        let mut set = match base {
            Some(largest) => reach[largest].clone(),
            None => HashSet::new(),
        };
        for &successor in &successors[index] {
            if Some(successor) != base {
                debug_assert!(
                    successor < index,
                    "Tarjan pop order must finalize every successor SCC first"
                );
                set.extend(reach[successor].iter().copied());
            }
        }
        set.extend(members.iter().copied());
        reach.push(set);
    }
    reach
}

/// Builds the closure per file: `direct_includes` straight from the sorted/
/// dedup adjacency; `transitive_includes` = the SCC reach set minus the
/// seed itself (the BFS seed-exclusion convention), sorted.
fn assemble_closures(
    components: &[Vec<FileId>],
    reach: &[HashSet<FileId>],
    adjacency: &HashMap<FileId, Vec<FileId>>,
) -> HashMap<FileId, IncludeClosure> {
    let mut closures = HashMap::with_capacity(reach.len());
    for (index, members) in components.iter().enumerate() {
        for &member in members {
            let direct = adjacency.get(&member).cloned().unwrap_or_default();
            let mut transitive: Vec<FileId> = reach[index]
                .iter()
                .copied()
                .filter(|&file_id| file_id != member)
                .collect();
            transitive.sort();
            closures.insert(member, IncludeClosure::new(member, direct, transitive));
        }
    }
    closures
}
