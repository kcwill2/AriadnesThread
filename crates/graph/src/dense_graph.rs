//! Integer graphs for whole-model passes: nodes are dense `u32` indices,
//! edges live in one flat array (compressed sparse rows), and cycles collapse
//! through strongly connected components. A walk over this form touches no
//! strings and allocates nothing per node, which is what makes one walk per
//! symbol affordable in the stage-5 symbol pass.

use rayon::prelude::*;

/// Successor lists of nodes `0..node_count`, deduplicated and ascending.
pub(crate) struct Adjacency {
    /// `targets[offsets[node]..offsets[node + 1]]` are `node`'s successors.
    offsets: Vec<usize>,
    targets: Vec<u32>,
}

impl Adjacency {
    /// Build from `(source, target)` pairs; duplicates collapse.
    pub(crate) fn from_pairs(node_count: usize, mut pairs: Vec<(u32, u32)>) -> Self {
        pairs.par_sort_unstable();
        pairs.dedup();
        let mut offsets = vec![0usize; node_count + 1];
        for &(source, _) in &pairs {
            offsets[source as usize + 1] += 1;
        }
        for node in 0..node_count {
            offsets[node + 1] += offsets[node];
        }
        let targets = pairs.into_iter().map(|(_, target)| target).collect();
        Self { offsets, targets }
    }

    pub(crate) fn node_count(&self) -> usize {
        self.offsets.len() - 1
    }

    pub(crate) fn successors(&self, node: u32) -> &[u32] {
        let node = node as usize;
        &self.targets[self.offsets[node]..self.offsets[node + 1]]
    }
}

/// The strongly connected component of every node. Components are numbered
/// in completion order, so every edge between two components runs from the
/// higher number to the lower one.
pub(crate) struct Components {
    pub(crate) component_of: Vec<u32>,
    pub(crate) count: usize,
}

const UNSET: u32 = u32::MAX;

/// Working state of the iterative Tarjan walk.
struct TarjanState {
    discovery: Vec<u32>,
    lowest: Vec<u32>,
    component_of: Vec<u32>,
    /// Discovered nodes not yet assigned to a component.
    open: Vec<u32>,
    discovered: u32,
    components: u32,
}

impl TarjanState {
    fn discover(&mut self, node: u32) {
        self.discovery[node as usize] = self.discovered;
        self.lowest[node as usize] = self.discovered;
        self.discovered += 1;
        self.open.push(node);
    }

    /// Close `node` once all its successors are done: if it is the first
    /// discovered node of its component, pop the component off `open`.
    fn finish(&mut self, node: u32) {
        if self.lowest[node as usize] != self.discovery[node as usize] {
            return;
        }
        while let Some(member) = self.open.pop() {
            self.component_of[member as usize] = self.components;
            if member == node {
                break;
            }
        }
        self.components += 1;
    }
}

/// Tarjan's algorithm with an explicit frame stack (no recursion: real call
/// graphs chain far deeper than the thread stack allows).
pub(crate) fn strongly_connected_components(adjacency: &Adjacency) -> Components {
    let node_count = adjacency.node_count();
    let mut state = TarjanState {
        discovery: vec![UNSET; node_count],
        lowest: vec![0; node_count],
        component_of: vec![UNSET; node_count],
        open: Vec::new(),
        discovered: 0,
        components: 0,
    };
    let mut frames: Vec<(u32, usize)> = Vec::new();
    for root in 0..node_count as u32 {
        if state.discovery[root as usize] != UNSET {
            continue;
        }
        state.discover(root);
        frames.push((root, 0));
        while let Some(frame) = frames.last_mut() {
            let (node, position) = *frame;
            if let Some(&next) = adjacency.successors(node).get(position) {
                frame.1 += 1;
                if state.discovery[next as usize] == UNSET {
                    state.discover(next);
                    frames.push((next, 0));
                } else if state.component_of[next as usize] == UNSET {
                    let seen = state.discovery[next as usize];
                    state.lowest[node as usize] = state.lowest[node as usize].min(seen);
                }
                continue;
            }
            frames.pop();
            if let Some(&(parent, _)) = frames.last() {
                let reached = state.lowest[node as usize];
                state.lowest[parent as usize] = state.lowest[parent as usize].min(reached);
            }
            state.finish(node);
        }
    }
    Components {
        component_of: state.component_of,
        count: state.components as usize,
    }
}

/// The graph of components: one node per component, an edge wherever any
/// member of one has a successor in another. Acyclic by construction.
pub(crate) fn condensation(adjacency: &Adjacency, components: &Components) -> Adjacency {
    let component_of = &components.component_of;
    let mut pairs: Vec<(u32, u32)> = Vec::new();
    for node in 0..adjacency.node_count() as u32 {
        let source = component_of[node as usize];
        for &next in adjacency.successors(node) {
            let target = component_of[next as usize];
            if source != target {
                pairs.push((source, target));
            }
        }
    }
    Adjacency::from_pairs(components.count, pairs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs_become_sorted_deduplicated_successor_lists() {
        let adjacency = Adjacency::from_pairs(4, vec![(2, 1), (0, 3), (2, 1), (2, 0)]);
        assert_eq!(adjacency.node_count(), 4);
        assert_eq!(adjacency.successors(0), &[3]);
        assert_eq!(adjacency.successors(1), &[] as &[u32]);
        assert_eq!(adjacency.successors(2), &[0, 1]);
    }

    #[test]
    fn a_cycle_collapses_to_one_component_and_edges_run_downward() {
        // 0 -> 1 -> 2 -> 0 is a cycle; 3 uses it; 4 stands alone.
        let adjacency = Adjacency::from_pairs(5, vec![(0, 1), (1, 2), (2, 0), (3, 0), (3, 3)]);
        let components = strongly_connected_components(&adjacency);
        let of = &components.component_of;
        assert_eq!(components.count, 3);
        assert_eq!(of[0], of[1]);
        assert_eq!(of[1], of[2]);
        assert_ne!(of[3], of[0]);
        assert_ne!(of[4], of[0]);
        assert_ne!(of[4], of[3]);

        let condensed = condensation(&adjacency, &components);
        assert_eq!(condensed.successors(of[3]), &[of[0]]);
        assert!(condensed.successors(of[0]).is_empty(), "the cycle's own edges vanish");
        assert!(of[3] > of[0], "edges run from the higher component number down");
    }

    #[test]
    fn a_long_chain_does_not_overflow_the_stack() {
        let length = 300_000u32;
        let pairs = (0..length - 1).map(|node| (node, node + 1)).collect();
        let adjacency = Adjacency::from_pairs(length as usize, pairs);
        let components = strongly_connected_components(&adjacency);
        assert_eq!(components.count, length as usize);
    }
}
