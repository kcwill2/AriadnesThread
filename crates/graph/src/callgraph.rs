use serde::{Deserialize, Serialize};
use ariadne_index::types::{FileLocation, SymbolId};
use ariadne_repo::FileId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallNode {
    pub symbol_id: SymbolId,
    pub name: String,
    pub owning_file: FileId,
    pub location: Option<FileLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallEdge {
    pub caller: SymbolId,
    pub callee: SymbolId,
    pub call_count: usize,
    pub call_locations: Vec<FileLocation>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallGraph {
    pub nodes: Vec<CallNode>,
    pub edges: Vec<CallEdge>,
}

impl CallGraph {
    pub fn build_from_index(index_result: &ariadne_index::IndexResult, model: &crate::ReducedSemanticModel) -> Self {
        let mut nodes: std::collections::BTreeMap<SymbolId, CallNode> = std::collections::BTreeMap::new();
        let mut edges: std::collections::BTreeMap<(SymbolId, SymbolId), CallEdge> = std::collections::BTreeMap::new();

        for tu in &index_result.tu_analyses {
            for site in &tu.call_sites {
                let caller = site.caller_symbol.clone();
                
                let callee_opt = model.symbols.values().find(|sym| sym.name == site.callee_name || sym.id.0 == site.callee_name);
                
                if let Some(callee_sym) = callee_opt {
                    let callee = callee_sym.id.clone();
                    
                    let edge = edges.entry((caller.clone(), callee.clone())).or_insert_with(|| CallEdge {
                        caller: caller.clone(),
                        callee: callee.clone(),
                        call_count: 0,
                        call_locations: Vec::new(),
                    });
                    
                    edge.call_count += 1;
                    if !edge.call_locations.contains(&site.location) {
                        edge.call_locations.push(site.location.clone());
                    }

                    // Add to nodes if not present
                    if !nodes.contains_key(&caller) {
                        if let Some(record) = model.symbols.get(&caller) {
                            nodes.insert(caller.clone(), CallNode {
                                symbol_id: caller.clone(),
                                name: record.name.clone(),
                                owning_file: record.defining_file.clone().unwrap_or_else(|| ariadne_repo::FileId::from_relative_path("unknown")),
                                location: None,
                            });
                        }
                    }
                    if !nodes.contains_key(&callee) {
                        nodes.insert(callee.clone(), CallNode {
                            symbol_id: callee.clone(),
                            name: callee_sym.name.clone(),
                            owning_file: callee_sym.defining_file.clone().unwrap_or_else(|| ariadne_repo::FileId::from_relative_path("unknown")),
                            location: None,
                        });
                    }
                }
            }
        }
        
        for (sym_id, record) in &model.symbols {
            if record.kind == ariadne_index::types::SymbolKind::Function && !nodes.contains_key(sym_id) {
                nodes.insert(sym_id.clone(), CallNode {
                    symbol_id: sym_id.clone(),
                    name: record.name.clone(),
                    owning_file: record.defining_file.clone().unwrap_or_else(|| ariadne_repo::FileId::from_relative_path("unknown")),
                    location: None,
                });
            }
        }

        Self {
            nodes: nodes.into_values().collect(),
            edges: edges.into_values().collect(),
        }
    }

    pub fn callers_of(&self, symbol: &SymbolId) -> Vec<&CallEdge> {
        self.edges.iter().filter(|e| e.callee == *symbol).collect()
    }

    pub fn callees_of(&self, symbol: &SymbolId) -> Vec<&CallEdge> {
        self.edges.iter().filter(|e| e.caller == *symbol).collect()
    }

    pub fn detect_recursion_cycles(&self) -> Vec<Vec<SymbolId>> {
        use std::collections::{HashSet, HashMap};

        let mut adj: HashMap<SymbolId, Vec<SymbolId>> = HashMap::new();
        for edge in &self.edges {
            adj.entry(edge.caller.clone()).or_default().push(edge.callee.clone());
        }

        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut path = Vec::new();

        fn dfs(
            node: &SymbolId,
            adj: &HashMap<SymbolId, Vec<SymbolId>>,
            visited: &mut HashSet<SymbolId>,
            path: &mut Vec<SymbolId>,
            cycles: &mut Vec<Vec<SymbolId>>
        ) {
            if let Some(idx) = path.iter().position(|x| x == node) {
                cycles.push(path[idx..].to_vec());
                return;
            }

            if visited.contains(node) {
                return;
            }

            visited.insert(node.clone());
            path.push(node.clone());

            if let Some(neighbors) = adj.get(node) {
                for neighbor in neighbors {
                    dfs(neighbor, adj, visited, path, cycles);
                }
            }

            path.pop();
        }

        let mut keys: Vec<_> = adj.keys().cloned().collect();
        keys.sort();

        for node in &keys {
            if !visited.contains(node) {
                dfs(node, &adj, &mut visited, &mut path, &mut cycles);
            }
        }

        for cycle in &mut cycles {
            let min_idx = cycle.iter().enumerate().min_by(|(_, a), (_, b)| a.cmp(b)).unwrap().0;
            cycle.rotate_left(min_idx);
        }
        cycles.sort();
        cycles.dedup();

        cycles
    }

    pub fn shortest_call_path(&self, from: &SymbolId, to: &SymbolId) -> Option<Vec<SymbolId>> {
        use std::collections::{VecDeque, HashSet, HashMap};

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent: HashMap<SymbolId, SymbolId> = HashMap::new();

        queue.push_back(from.clone());
        visited.insert(from.clone());

        while let Some(current) = queue.pop_front() {
            if current == *to {
                let mut path = Vec::new();
                let mut curr = current.clone();
                while let Some(p) = parent.get(&curr) {
                    path.push(curr.clone());
                    curr = p.clone();
                }
                path.push(from.clone());
                path.reverse();
                return Some(path);
            }

            let callees = self.callees_of(&current);
            let mut neighbors: Vec<_> = callees.iter().map(|e| e.callee.clone()).collect();
            neighbors.sort();

            for next in neighbors {
                if !visited.contains(&next) {
                    visited.insert(next.clone());
                    parent.insert(next.clone(), current.clone());
                    queue.push_back(next);
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_index::types::{TuAnalysis, SymbolRecord, SymbolKind, CallSite};
    use crate::ReducedSemanticModel;
    use std::path::PathBuf;

    fn setup() -> (ariadne_index::IndexResult, ReducedSemanticModel) {
        let f1 = FileId::from_relative_path("main.rs");
        let sym_a = SymbolId::new("A");
        let sym_b = SymbolId::new("B");
        let sym_c = SymbolId::new("C");

        let tu = TuAnalysis {
            tu_file_id: f1.clone(),
            context_name: "test".into(),
            symbols: vec![
                SymbolRecord {
                    id: sym_a.clone(),
                    name: "A".into(),
                    kind: SymbolKind::Function,
                    owning_file: f1.clone(),
                    declarations: vec![],
                    definition: None,
                },
                SymbolRecord {
                    id: sym_b.clone(),
                    name: "B".into(),
                    kind: SymbolKind::Function,
                    owning_file: f1.clone(),
                    declarations: vec![],
                    definition: None,
                },
                SymbolRecord {
                    id: sym_c.clone(),
                    name: "C".into(),
                    kind: SymbolKind::Function,
                    owning_file: f1.clone(),
                    declarations: vec![],
                    definition: None,
                },
            ],
            declarations: vec![],
            edges: vec![],
            references: vec![],
            call_sites: vec![
                CallSite {
                    caller_symbol: sym_a.clone(),
                    callee_name: "B".into(),
                    location: FileLocation { file_id: f1.clone(), line: 10, column: 5 },
                },
                CallSite {
                    caller_symbol: sym_b.clone(),
                    callee_name: "C".into(),
                    location: FileLocation { file_id: f1.clone(), line: 20, column: 5 },
                },
                CallSite {
                    caller_symbol: sym_c.clone(),
                    callee_name: "A".into(), // cycle A -> B -> C -> A
                    location: FileLocation { file_id: f1.clone(), line: 30, column: 5 },
                },
                CallSite {
                    caller_symbol: sym_c.clone(),
                    callee_name: "C".into(), // self recursion
                    location: FileLocation { file_id: f1.clone(), line: 31, column: 5 },
                }
            ],
        };

        let index_result = ariadne_index::IndexResult::new(PathBuf::from("/test"), vec![tu]);
        let model = ReducedSemanticModel::reduce(&index_result);
        (index_result, model)
    }

    #[test]
    fn test_callgraph_build() {
        let (index, model) = setup();
        let graph = CallGraph::build_from_index(&index, &model);
        
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 4);
    }

    #[test]
    fn test_callers_and_callees() {
        let (index, model) = setup();
        let graph = CallGraph::build_from_index(&index, &model);
        
        let callers_of_b = graph.callers_of(&SymbolId::new("B"));
        assert_eq!(callers_of_b.len(), 1);
        assert_eq!(callers_of_b[0].caller, SymbolId::new("A"));

        let callees_of_b = graph.callees_of(&SymbolId::new("B"));
        assert_eq!(callees_of_b.len(), 1);
        assert_eq!(callees_of_b[0].callee, SymbolId::new("C"));
    }

    #[test]
    fn test_recursion_cycles() {
        let (index, model) = setup();
        let graph = CallGraph::build_from_index(&index, &model);
        
        let cycles = graph.detect_recursion_cycles();
        assert_eq!(cycles.len(), 2);
    }

    #[test]
    fn test_shortest_call_path() {
        let (index, model) = setup();
        let graph = CallGraph::build_from_index(&index, &model);
        
        let path = graph.shortest_call_path(&SymbolId::new("A"), &SymbolId::new("C"));
        assert!(path.is_some());
        assert_eq!(path.unwrap(), vec![SymbolId::new("A"), SymbolId::new("B"), SymbolId::new("C")]);
    }
}
