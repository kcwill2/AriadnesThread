use ariadne_index::types::{FileLocation, SymbolId};
use ariadne_repo::FileId;
use serde::{Deserialize, Serialize};

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

struct CalleeLookup<'m> {
    by_exact_name: std::collections::HashMap<&'m str, &'m crate::ReducedSymbolRecord>,
    by_id_suffix: std::collections::HashMap<&'m str, &'m crate::ReducedSymbolRecord>,
}

impl<'m> CalleeLookup<'m> {
    fn from_model(model: &'m crate::ReducedSemanticModel) -> Self {
        let mut by_exact_name: std::collections::HashMap<&str, &crate::ReducedSymbolRecord> =
            std::collections::HashMap::new();
        let mut by_id_suffix: std::collections::HashMap<&str, &crate::ReducedSymbolRecord> =
            std::collections::HashMap::new();

        for sym in model.symbols.values() {
            by_exact_name.insert(sym.id.0.as_str(), sym);
            by_exact_name.entry(sym.name.as_str()).or_insert(sym);

            let mut current = sym.id.0.as_str();
            while let Some(idx) = current.find("::") {
                current = &current[idx + 2..];
                by_id_suffix.entry(current).or_insert(sym);
            }
        }

        Self {
            by_exact_name,
            by_id_suffix,
        }
    }

    fn resolve(&self, callee_name: &str) -> Option<&'m crate::ReducedSymbolRecord> {
        self.by_exact_name
            .get(callee_name)
            .copied()
            .or_else(|| self.by_id_suffix.get(callee_name).copied())
            .or_else(|| {
                let mut current = callee_name;
                while let Some(idx) = current.find("::") {
                    current = &current[idx + 2..];
                    if let Some(&sym) = self.by_exact_name.get(current) {
                        return Some(sym);
                    }
                }
                None
            })
    }
}

fn call_node(
    symbol_id: SymbolId,
    name: String,
    defining_file: Option<ariadne_repo::FileId>,
) -> CallNode {
    CallNode {
        symbol_id,
        name,
        owning_file: defining_file
            .unwrap_or_else(|| ariadne_repo::FileId::from_relative_path("unknown")),
        location: None,
    }
}

fn record_call_site(
    nodes: &mut std::collections::BTreeMap<SymbolId, CallNode>,
    edges: &mut std::collections::BTreeMap<(SymbolId, SymbolId), CallEdge>,
    model: &crate::ReducedSemanticModel,
    site: &ariadne_index::types::CallSite,
    callee_sym: &crate::ReducedSymbolRecord,
) {
    let caller = site.caller_symbol.clone();
    let callee = callee_sym.id.clone();

    let edge = edges
        .entry((caller.clone(), callee.clone()))
        .or_insert_with(|| CallEdge {
            caller: caller.clone(),
            callee: callee.clone(),
            call_count: 0,
            call_locations: Vec::new(),
        });

    edge.call_count += 1;
    if !edge.call_locations.contains(&site.location) {
        edge.call_locations.push(site.location.clone());
    }

    if !nodes.contains_key(&caller) {
        if let Some(record) = model.symbols.get(&caller) {
            nodes.insert(
                caller.clone(),
                call_node(
                    caller.clone(),
                    record.name.clone(),
                    record.defining_file.clone(),
                ),
            );
        }
    }
    if !nodes.contains_key(&callee) {
        nodes.insert(
            callee.clone(),
            call_node(
                callee.clone(),
                callee_sym.name.clone(),
                callee_sym.defining_file.clone(),
            ),
        );
    }
}

fn seed_uncalled_function_nodes(
    nodes: &mut std::collections::BTreeMap<SymbolId, CallNode>,
    model: &crate::ReducedSemanticModel,
) {
    for (sym_id, record) in &model.symbols {
        if record.kind == ariadne_index::types::SymbolKind::Function && !nodes.contains_key(sym_id)
        {
            nodes.insert(
                sym_id.clone(),
                call_node(
                    sym_id.clone(),
                    record.name.clone(),
                    record.defining_file.clone(),
                ),
            );
        }
    }
}

impl CallGraph {
    pub fn build_from_index(
        index_result: &ariadne_index::IndexResult,
        model: &crate::ReducedSemanticModel,
    ) -> Self {
        let mut nodes: std::collections::BTreeMap<SymbolId, CallNode> =
            std::collections::BTreeMap::new();
        let mut edges: std::collections::BTreeMap<(SymbolId, SymbolId), CallEdge> =
            std::collections::BTreeMap::new();

        let lookup = CalleeLookup::from_model(model);

        for tu in &index_result.tu_analyses {
            for site in &tu.call_sites {
                if let Some(callee_sym) = lookup.resolve(site.callee_name.as_str()) {
                    record_call_site(&mut nodes, &mut edges, model, site, callee_sym);
                }
            }
        }

        seed_uncalled_function_nodes(&mut nodes, model);

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
        use std::collections::{HashMap, HashSet};

        let mut adj: HashMap<SymbolId, Vec<SymbolId>> = HashMap::new();
        for edge in &self.edges {
            adj.entry(edge.caller.clone())
                .or_default()
                .push(edge.callee.clone());
        }

        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut path = Vec::new();

        fn dfs(
            node: &SymbolId,
            adj: &HashMap<SymbolId, Vec<SymbolId>>,
            visited: &mut HashSet<SymbolId>,
            path: &mut Vec<SymbolId>,
            cycles: &mut Vec<Vec<SymbolId>>,
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
            let min_idx = cycle
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.cmp(b))
                .expect("cycles are pushed as path[idx..] slices, which are never empty")
                .0;
            cycle.rotate_left(min_idx);
        }
        cycles.sort();
        cycles.dedup();

        cycles
    }

    pub fn shortest_call_path(&self, from: &SymbolId, to: &SymbolId) -> Option<Vec<SymbolId>> {
        use std::collections::{HashMap, HashSet, VecDeque};

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
    use crate::ReducedSemanticModel;
    use ariadne_index::types::{CallSite, SymbolKind, SymbolRecord, TuAnalysis};
    use std::path::PathBuf;

    fn setup() -> (ariadne_index::IndexResult, ReducedSemanticModel) {
        let f1 = FileId::from_relative_path("main.rs");
        let sym_a = SymbolId::new("A");
        let sym_b = SymbolId::new("B");
        let sym_c = SymbolId::new("C");

        let tu = TuAnalysis {
            tu_file_id: f1.clone(),
            context_name: "test".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
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
                    location: FileLocation {
                        file_id: f1.clone(),
                        line: 10,
                        column: 5,
                    },
                },
                CallSite {
                    caller_symbol: sym_b.clone(),
                    callee_name: "C".into(),
                    location: FileLocation {
                        file_id: f1.clone(),
                        line: 20,
                        column: 5,
                    },
                },
                CallSite {
                    caller_symbol: sym_c.clone(),
                    callee_name: "A".into(), // cycle A -> B -> C -> A
                    location: FileLocation {
                        file_id: f1.clone(),
                        line: 30,
                        column: 5,
                    },
                },
                CallSite {
                    caller_symbol: sym_c.clone(),
                    callee_name: "C".into(), // self recursion
                    location: FileLocation {
                        file_id: f1.clone(),
                        line: 31,
                        column: 5,
                    },
                },
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
        assert_eq!(
            path.unwrap(),
            vec![SymbolId::new("A"), SymbolId::new("B"), SymbolId::new("C")]
        );
    }

    #[test]
    fn test_callgraph_build_qualified_names() {
        let f1 = FileId::from_relative_path("main.rs");
        let sym_caller = SymbolId::new("caller");
        let sym_callee = SymbolId::new("ns::MyClass::callee");

        let tu = TuAnalysis {
            tu_file_id: f1.clone(),
            context_name: "test".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![
                SymbolRecord {
                    id: sym_caller.clone(),
                    name: "caller".into(),
                    kind: SymbolKind::Function,
                    owning_file: f1.clone(),
                    declarations: vec![],
                    definition: None,
                },
                SymbolRecord {
                    id: sym_callee.clone(),
                    name: "callee".into(), // The name might be just "callee"
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
                    caller_symbol: sym_caller.clone(),
                    callee_name: "callee".into(), // Caller just says "callee()"
                    location: FileLocation {
                        file_id: f1.clone(),
                        line: 10,
                        column: 5,
                    },
                },
                CallSite {
                    caller_symbol: sym_caller.clone(),
                    callee_name: "MyClass::callee".into(), // Caller uses fully qualified name
                    location: FileLocation {
                        file_id: f1.clone(),
                        line: 11,
                        column: 5,
                    },
                },
            ],
        };

        let index_result = ariadne_index::IndexResult::new(PathBuf::from("/test"), vec![tu]);
        let model = ReducedSemanticModel::reduce(&index_result);

        let graph = CallGraph::build_from_index(&index_result, &model);

        let callers = graph.callers_of(&sym_callee);
        assert_eq!(
            callers.len(),
            1,
            "Should map both call sites to the same callee"
        );
        assert_eq!(callers[0].call_count, 2, "Should have 2 calls");
    }

    #[test]
    fn test_callgraph_resolution_priority() {
        let f1 = FileId::from_relative_path("main.rs");
        let sym_caller = SymbolId::new("caller");
        // We have two symbols. One matches exactly by name, one matches by suffix.
        let sym_exact = SymbolId::new("exact::method");
        let sym_suffix = SymbolId::new("other::exact::method");

        let tu = TuAnalysis {
            tu_file_id: f1.clone(),
            context_name: "test".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![
                SymbolRecord {
                    id: sym_exact.clone(),
                    name: "exact::method".into(),
                    kind: SymbolKind::Function,
                    owning_file: f1.clone(),
                    declarations: vec![],
                    definition: None,
                },
                SymbolRecord {
                    id: sym_suffix.clone(),
                    name: "method".into(),
                    kind: SymbolKind::Function,
                    owning_file: f1.clone(),
                    declarations: vec![],
                    definition: None,
                },
            ],
            declarations: vec![],
            edges: vec![],
            references: vec![],
            call_sites: vec![CallSite {
                caller_symbol: sym_caller.clone(),
                callee_name: "exact::method".into(), // exact match for sym_exact by ID/Name
                location: FileLocation {
                    file_id: f1.clone(),
                    line: 10,
                    column: 5,
                },
            }],
        };

        let index_result = ariadne_index::IndexResult::new(PathBuf::from("/test"), vec![tu]);
        let model = ReducedSemanticModel::reduce(&index_result);

        let graph = CallGraph::build_from_index(&index_result, &model);

        // Because of HashMap random order in the old implementation, it could pick sym_suffix
        // since `callee_name` ("exact::method") ends with `::sym.name` ("::method").
        // We want to ensure it strictly picks exact matches first.
        let callers = graph.callers_of(&sym_exact);
        assert_eq!(
            callers.len(),
            1,
            "Should prefer exact match over suffix match"
        );
    }
}
