use crate::{CallGraph, IncludeGraph, ReducedSemanticModel};
use ariadne_index::types::SymbolId;
use ariadne_repo::FileId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraversalDirection {
    Incoming,
    Outgoing,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubgraphSelection<T> {
    pub nodes: Vec<T>,
    pub edges: Vec<(T, T)>,
}

impl IncludeGraph {
    pub fn traverse_neighborhood(
        &self,
        center: FileId,
        depth: usize,
        direction: TraversalDirection,
    ) -> SubgraphSelection<FileId> {
        use std::collections::{HashSet, VecDeque};

        let mut visited_nodes = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((center, 0));
        visited_nodes.insert(center);

        while let Some((curr, d)) = queue.pop_front() {
            if d >= depth {
                continue;
            }

            for edge in &self.edges {
                if let Some(dest) = edge.destination {
                    let mut next_node = None;

                    if (direction == TraversalDirection::Outgoing
                        || direction == TraversalDirection::Both)
                        && edge.source == curr
                    {
                        next_node = Some(dest);
                    }

                    if (direction == TraversalDirection::Incoming
                        || direction == TraversalDirection::Both)
                        && dest == curr
                    {
                        next_node = Some(edge.source);
                    }

                    if let Some(n) = next_node {
                        if !visited_nodes.contains(&n) {
                            visited_nodes.insert(n);
                            queue.push_back((n, d + 1));
                        }
                    }
                }
            }
        }

        let mut edges = HashSet::new();
        for edge in &self.edges {
            if let Some(dest) = edge.destination {
                if visited_nodes.contains(&edge.source) && visited_nodes.contains(&dest) {
                    edges.insert((edge.source, dest));
                }
            }
        }

        let mut nodes: Vec<_> = visited_nodes.into_iter().collect();
        nodes.sort();
        let mut edges_vec: Vec<_> = edges.into_iter().collect();
        edges_vec.sort();

        SubgraphSelection {
            nodes,
            edges: edges_vec,
        }
    }

    pub fn shortest_include_path(&self, from: FileId, to: FileId) -> Option<Vec<FileId>> {
        use std::collections::{HashMap, HashSet, VecDeque};

        if from == to {
            return Some(vec![from]);
        }

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent = HashMap::new();

        queue.push_back(from);
        visited.insert(from);

        while let Some(curr) = queue.pop_front() {
            if curr == to {
                let mut path = Vec::new();
                let mut current = curr;
                while let Some(&p) = parent.get(&current) {
                    path.push(current);
                    current = p;
                }
                path.push(from);
                path.reverse();
                return Some(path);
            }

            for edge in &self.edges {
                if edge.source == curr {
                    if let Some(dest) = edge.destination {
                        if !visited.contains(&dest) {
                            visited.insert(dest);
                            parent.insert(dest, curr);
                            queue.push_back(dest);
                        }
                    }
                }
            }
        }

        None
    }

    pub fn get_entrypoints(&self, limit: usize) -> (Vec<FileId>, Vec<FileId>, Vec<FileId>) {
        use std::collections::HashMap;

        let mut in_degree = HashMap::new();
        let mut out_degree = HashMap::new();
        let mut nodes = std::collections::HashSet::new();

        // Also include all files registered
        for &id in self.file_paths.keys() {
            nodes.insert(id);
        }

        for edge in &self.edges {
            nodes.insert(edge.source);
            if let Some(dest) = edge.destination {
                nodes.insert(dest);
                *out_degree.entry(edge.source).or_insert(0) += 1;
                *in_degree.entry(dest).or_insert(0) += 1;
            }
        }

        let mut all_nodes: Vec<_> = nodes.into_iter().collect();
        all_nodes.sort();

        let mut roots = Vec::new();
        let mut leaves = Vec::new();
        let mut hubs = all_nodes.clone();

        for &n in &all_nodes {
            if *in_degree.get(&n).unwrap_or(&0) == 0 {
                roots.push(n);
            }
            if *out_degree.get(&n).unwrap_or(&0) == 0 {
                leaves.push(n);
            }
        }

        hubs.sort_by(|a, b| {
            let deg_a = in_degree.get(a).unwrap_or(&0) + out_degree.get(a).unwrap_or(&0);
            let deg_b = in_degree.get(b).unwrap_or(&0) + out_degree.get(b).unwrap_or(&0);
            deg_b.cmp(&deg_a).then_with(|| a.cmp(b))
        });

        roots.truncate(limit);
        leaves.truncate(limit);
        hubs.truncate(limit);

        (roots, leaves, hubs)
    }
}

impl ReducedSemanticModel {
    pub fn traverse_neighborhood(
        &self,
        center: &SymbolId,
        depth: usize,
        direction: TraversalDirection,
    ) -> SubgraphSelection<SymbolId> {
        use std::collections::{HashSet, VecDeque};

        let mut visited_nodes = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((center.clone(), 0));
        visited_nodes.insert(center.clone());

        while let Some((curr, d)) = queue.pop_front() {
            if d >= depth {
                continue;
            }

            for edge in &self.edges {
                let mut next_node = None;

                if (direction == TraversalDirection::Outgoing
                    || direction == TraversalDirection::Both)
                    && edge.src_symbol == curr
                {
                    next_node = Some(edge.dst_symbol.clone());
                }

                if (direction == TraversalDirection::Incoming
                    || direction == TraversalDirection::Both)
                    && edge.dst_symbol == curr
                {
                    next_node = Some(edge.src_symbol.clone());
                }

                if let Some(n) = next_node {
                    if !visited_nodes.contains(&n) {
                        visited_nodes.insert(n.clone());
                        queue.push_back((n, d + 1));
                    }
                }
            }
        }

        let mut edges = HashSet::new();
        for edge in &self.edges {
            if visited_nodes.contains(&edge.src_symbol) && visited_nodes.contains(&edge.dst_symbol)
            {
                edges.insert((edge.src_symbol.clone(), edge.dst_symbol.clone()));
            }
        }

        let mut nodes: Vec<_> = visited_nodes.into_iter().collect();
        nodes.sort();
        let mut edges_vec: Vec<_> = edges.into_iter().collect();
        edges_vec.sort();

        SubgraphSelection {
            nodes,
            edges: edges_vec,
        }
    }

    pub fn shortest_dependency_path(
        &self,
        from: &SymbolId,
        to: &SymbolId,
    ) -> Option<Vec<SymbolId>> {
        use std::collections::{HashMap, HashSet, VecDeque};

        if from == to {
            return Some(vec![from.clone()]);
        }

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent: HashMap<SymbolId, SymbolId> = HashMap::new();

        queue.push_back(from.clone());
        visited.insert(from.clone());

        while let Some(curr) = queue.pop_front() {
            if curr == *to {
                let mut path = Vec::new();
                let mut current = curr;
                while let Some(p) = parent.get(&current) {
                    path.push(current.clone());
                    current = p.clone();
                }
                path.push(from.clone());
                path.reverse();
                return Some(path);
            }

            for edge in &self.edges {
                if edge.src_symbol == curr {
                    let dest = edge.dst_symbol.clone();
                    if !visited.contains(&dest) {
                        visited.insert(dest.clone());
                        parent.insert(dest.clone(), curr.clone());
                        queue.push_back(dest);
                    }
                }
            }
        }

        None
    }

    pub fn get_entrypoints(&self, limit: usize) -> (Vec<SymbolId>, Vec<SymbolId>, Vec<SymbolId>) {
        use std::collections::HashMap;

        let mut in_degree = HashMap::new();
        let mut out_degree = HashMap::new();
        let mut nodes = std::collections::HashSet::new();

        for id in self.symbols.keys() {
            nodes.insert(id.clone());
        }

        for edge in &self.edges {
            nodes.insert(edge.src_symbol.clone());
            nodes.insert(edge.dst_symbol.clone());
            *out_degree.entry(edge.src_symbol.clone()).or_insert(0) += 1;
            *in_degree.entry(edge.dst_symbol.clone()).or_insert(0) += 1;
        }

        let mut all_nodes: Vec<_> = nodes.into_iter().collect();
        all_nodes.sort();

        let mut roots = Vec::new();
        let mut leaves = Vec::new();
        let mut hubs = all_nodes.clone();

        for n in &all_nodes {
            if *in_degree.get(n).unwrap_or(&0) == 0 {
                roots.push(n.clone());
            }
            if *out_degree.get(n).unwrap_or(&0) == 0 {
                leaves.push(n.clone());
            }
        }

        hubs.sort_by(|a, b| {
            let deg_a = in_degree.get(a).unwrap_or(&0) + out_degree.get(a).unwrap_or(&0);
            let deg_b = in_degree.get(b).unwrap_or(&0) + out_degree.get(b).unwrap_or(&0);
            deg_b.cmp(&deg_a).then_with(|| a.cmp(b))
        });

        roots.truncate(limit);
        leaves.truncate(limit);
        hubs.truncate(limit);

        (roots, leaves, hubs)
    }
}

impl CallGraph {
    pub fn traverse_neighborhood(
        &self,
        center: &SymbolId,
        depth: usize,
        direction: TraversalDirection,
    ) -> SubgraphSelection<SymbolId> {
        use std::collections::{HashSet, VecDeque};

        let mut visited_nodes = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((center.clone(), 0));
        visited_nodes.insert(center.clone());

        while let Some((curr, d)) = queue.pop_front() {
            if d >= depth {
                continue;
            }

            for edge in &self.edges {
                let mut next_node = None;

                if (direction == TraversalDirection::Outgoing
                    || direction == TraversalDirection::Both)
                    && edge.caller == curr
                {
                    next_node = Some(edge.callee.clone());
                }

                if (direction == TraversalDirection::Incoming
                    || direction == TraversalDirection::Both)
                    && edge.callee == curr
                {
                    next_node = Some(edge.caller.clone());
                }

                if let Some(n) = next_node {
                    if !visited_nodes.contains(&n) {
                        visited_nodes.insert(n.clone());
                        queue.push_back((n, d + 1));
                    }
                }
            }
        }

        let mut edges = HashSet::new();
        for edge in &self.edges {
            if visited_nodes.contains(&edge.caller) && visited_nodes.contains(&edge.callee) {
                edges.insert((edge.caller.clone(), edge.callee.clone()));
            }
        }

        let mut nodes: Vec<_> = visited_nodes.into_iter().collect();
        nodes.sort();
        let mut edges_vec: Vec<_> = edges.into_iter().collect();
        edges_vec.sort();

        SubgraphSelection {
            nodes,
            edges: edges_vec,
        }
    }

    pub fn get_entrypoints(&self, limit: usize) -> (Vec<SymbolId>, Vec<SymbolId>, Vec<SymbolId>) {
        use std::collections::HashMap;

        let mut in_degree = HashMap::new();
        let mut out_degree = HashMap::new();
        let mut nodes = std::collections::HashSet::new();

        for node in &self.nodes {
            nodes.insert(node.symbol_id.clone());
        }

        for edge in &self.edges {
            nodes.insert(edge.caller.clone());
            nodes.insert(edge.callee.clone());
            *out_degree.entry(edge.caller.clone()).or_insert(0) += 1;
            *in_degree.entry(edge.callee.clone()).or_insert(0) += 1;
        }

        let mut all_nodes: Vec<_> = nodes.into_iter().collect();
        all_nodes.sort();

        let mut roots = Vec::new();
        let mut leaves = Vec::new();
        let mut hubs = all_nodes.clone();

        for n in &all_nodes {
            if *in_degree.get(n).unwrap_or(&0) == 0 {
                roots.push(n.clone());
            }
            if *out_degree.get(n).unwrap_or(&0) == 0 {
                leaves.push(n.clone());
            }
        }

        hubs.sort_by(|a, b| {
            let deg_a = in_degree.get(a).unwrap_or(&0) + out_degree.get(a).unwrap_or(&0);
            let deg_b = in_degree.get(b).unwrap_or(&0) + out_degree.get(b).unwrap_or(&0);
            deg_b.cmp(&deg_a).then_with(|| a.cmp(b))
        });

        roots.truncate(limit);
        leaves.truncate(limit);
        hubs.truncate(limit);

        (roots, leaves, hubs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{IncludeEdge, IncludeKind, SourceLocation};
    use std::path::PathBuf;

    #[test]
    fn test_include_graph_traversal() {
        let mut graph = IncludeGraph::new();
        let f1 = FileId(1);
        let f2 = FileId(2);
        let f3 = FileId(3);
        graph.register_file(f1, PathBuf::from("a.h"));
        graph.register_file(f2, PathBuf::from("b.h"));
        graph.register_file(f3, PathBuf::from("c.h"));

        graph.add_edge(IncludeEdge {
            source: f1,
            destination: Some(f2),
            context_name: "test".into(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: "b.h".into(),
            resolved_path: None,
            language: crate::types::LanguageId::cpp(),
        });

        graph.add_edge(IncludeEdge {
            source: f2,
            destination: Some(f3),
            context_name: "test".into(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: "c.h".into(),
            resolved_path: None,
            language: crate::types::LanguageId::cpp(),
        });

        let hood = graph.traverse_neighborhood(f2, 1, TraversalDirection::Both);
        assert_eq!(hood.nodes.len(), 3);
        assert_eq!(hood.edges.len(), 2);

        let path = graph.shortest_include_path(f1, f3);
        assert_eq!(path, Some(vec![f1, f2, f3]));

        let (roots, leaves, hubs) = graph.get_entrypoints(1);
        assert_eq!(roots, vec![f1]);
        assert_eq!(leaves, vec![f3]);
        assert_eq!(hubs, vec![f2]); // f2 has degree 2
    }
}
