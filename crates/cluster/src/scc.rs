use ariadne_index::types::{SemanticEdge, SymbolId};
use std::cmp::min;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolScc {
    pub symbols: Vec<SymbolId>,
}

pub fn compute_symbol_sccs(symbols: &[SymbolId], edges: &[SemanticEdge]) -> Vec<SymbolScc> {
    let mut adj: HashMap<&SymbolId, Vec<&SymbolId>> = HashMap::new();
    for edge in edges {
        adj.entry(&edge.src_symbol).or_default().push(&edge.dst_symbol);
    }

    let mut index = 0;
    let mut indices: HashMap<&SymbolId, usize> = HashMap::new();
    let mut lowlink: HashMap<&SymbolId, usize> = HashMap::new();
    let mut on_stack: HashSet<&SymbolId> = HashSet::new();
    let mut stack: Vec<&SymbolId> = Vec::new();
    let mut sccs: Vec<SymbolScc> = Vec::new();

    for sym in symbols {
        if !indices.contains_key(sym) {
            strongconnect(
                sym,
                &adj,
                &mut index,
                &mut indices,
                &mut lowlink,
                &mut on_stack,
                &mut stack,
                &mut sccs,
            );
        }
    }

    sccs
}

fn strongconnect<'a>(
    v: &'a SymbolId,
    adj: &HashMap<&SymbolId, Vec<&'a SymbolId>>,
    index: &mut usize,
    indices: &mut HashMap<&'a SymbolId, usize>,
    lowlink: &mut HashMap<&'a SymbolId, usize>,
    on_stack: &mut HashSet<&'a SymbolId>,
    stack: &mut Vec<&'a SymbolId>,
    sccs: &mut Vec<SymbolScc>,
) {
    indices.insert(v, *index);
    lowlink.insert(v, *index);
    *index += 1;
    stack.push(v);
    on_stack.insert(v);

    if let Some(neighbors) = adj.get(v) {
        for w in neighbors {
            if !indices.contains_key(w) {
                strongconnect(w, adj, index, indices, lowlink, on_stack, stack, sccs);
                let new_lowlink = min(*lowlink.get(v).unwrap(), *lowlink.get(w).unwrap());
                lowlink.insert(v, new_lowlink);
            } else if on_stack.contains(w) {
                let new_lowlink = min(*lowlink.get(v).unwrap(), *indices.get(w).unwrap());
                lowlink.insert(v, new_lowlink);
            }
        }
    }

    if lowlink.get(v) == indices.get(v) {
        let mut scc = Vec::new();
        loop {
            let w = stack.pop().unwrap();
            on_stack.remove(w);
            scc.push(w.clone());
            if w == v {
                break;
            }
        }
        // To make deterministic, sort inside SCC if needed, but not strictly required
        // unless test demands it. We just leave it as is, or we could sort by symbol name.
        scc.sort_by(|a, b| a.0.cmp(&b.0));
        sccs.push(SymbolScc { symbols: scc });
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_index::types::DependencyKind;
    use ariadne_repo::FileId;

    fn edge(src: &str, dst: &str) -> SemanticEdge {
        SemanticEdge {
            src_symbol: SymbolId::new(src),
            dst_symbol: SymbolId::new(dst),
            kind: DependencyKind::Complete,
            src_file: FileId::from_relative_path("test"),
            dst_file: FileId::from_relative_path("test"),
        }
    }

    #[test]
    fn test_acyclic_sccs() {
        let syms = vec![
            SymbolId::new("A"),
            SymbolId::new("B"),
            SymbolId::new("C"),
        ];
        // A -> B -> C
        let edges = vec![
            edge("A", "B"),
            edge("B", "C"),
        ];

        let sccs = compute_symbol_sccs(&syms, &edges);
        assert_eq!(sccs.len(), 3);
        // Each should be in its own SCC, topologically sorted (leaf first) or reverse
        // Tarjan typically returns reverse topological order: C, B, A
        let mut lengths: Vec<usize> = sccs.iter().map(|scc| scc.symbols.len()).collect();
        lengths.sort();
        assert_eq!(lengths, vec![1, 1, 1]);
    }

    #[test]
    fn test_cyclic_sccs() {
        let syms = vec![
            SymbolId::new("A"),
            SymbolId::new("B"),
            SymbolId::new("C"),
            SymbolId::new("D"),
        ];
        // A -> B -> C -> A (Cycle)
        // D -> C (No cycle)
        let edges = vec![
            edge("A", "B"),
            edge("B", "C"),
            edge("C", "A"),
            edge("D", "C"),
        ];

        let sccs = compute_symbol_sccs(&syms, &edges);
        assert_eq!(sccs.len(), 2);
        
        let mut lengths: Vec<usize> = sccs.iter().map(|scc| scc.symbols.len()).collect();
        lengths.sort();
        assert_eq!(lengths, vec![1, 3]);
        
        let cycle_scc = sccs.iter().find(|scc| scc.symbols.len() == 3).unwrap();
        assert!(cycle_scc.symbols.contains(&SymbolId::new("A")));
        assert!(cycle_scc.symbols.contains(&SymbolId::new("B")));
        assert!(cycle_scc.symbols.contains(&SymbolId::new("C")));
    }

    #[test]
    fn test_empty_input() {
        let syms = vec![];
        let edges = vec![];
        let sccs = compute_symbol_sccs(&syms, &edges);
        assert!(sccs.is_empty());
    }

    #[test]
    fn test_disconnected_components() {
        let syms = vec![
            SymbolId::new("A"),
            SymbolId::new("B"),
        ];
        let edges = vec![];
        let sccs = compute_symbol_sccs(&syms, &edges);
        assert_eq!(sccs.len(), 2);
    }

    #[test]
    fn test_self_loop() {
        let syms = vec![
            SymbolId::new("A"),
        ];
        let edges = vec![
            edge("A", "A"),
        ];
        let sccs = compute_symbol_sccs(&syms, &edges);
        assert_eq!(sccs.len(), 1);
        assert_eq!(sccs[0].symbols.len(), 1);
        assert_eq!(sccs[0].symbols[0], SymbolId::new("A"));
    }

    #[test]
    fn test_multiple_disjoint_sccs() {
        let syms = vec![
            SymbolId::new("A"),
            SymbolId::new("B"),
            SymbolId::new("C"),
            SymbolId::new("D"),
            SymbolId::new("E"),
        ];
        let edges = vec![
            edge("A", "B"),
            edge("B", "A"),
            edge("C", "D"),
            edge("D", "E"),
            edge("E", "C"),
        ];
        let mut sccs = compute_symbol_sccs(&syms, &edges);
        assert_eq!(sccs.len(), 2);
        sccs.sort_by_key(|scc| scc.symbols.len());
        assert_eq!(sccs[0].symbols.len(), 2);
        assert!(sccs[0].symbols.contains(&SymbolId::new("A")));
        assert!(sccs[0].symbols.contains(&SymbolId::new("B")));
        assert_eq!(sccs[1].symbols.len(), 3);
        assert!(sccs[1].symbols.contains(&SymbolId::new("C")));
        assert!(sccs[1].symbols.contains(&SymbolId::new("D")));
        assert!(sccs[1].symbols.contains(&SymbolId::new("E")));
    }

    #[test]
    fn test_connected_multi_scc_graph() {
        let syms = vec![
            SymbolId::new("A"),
            SymbolId::new("B"),
            SymbolId::new("C"),
            SymbolId::new("D"),
        ];
        let edges = vec![
            edge("A", "B"),
            edge("B", "A"),
            edge("B", "C"),
            edge("C", "D"),
            edge("D", "C"),
        ];
        let sccs = compute_symbol_sccs(&syms, &edges);
        assert_eq!(sccs.len(), 2);
        
        let mut lengths: Vec<usize> = sccs.iter().map(|scc| scc.symbols.len()).collect();
        lengths.sort();
        assert_eq!(lengths, vec![2, 2]);
    }
}
