//! Include and semantic graph structures.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct IncludeGraph {
    pub edges: HashMap<PathBuf, HashSet<PathBuf>>,
}

impl IncludeGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_include(&mut self, from: PathBuf, to: PathBuf) {
        self.edges.entry(from).or_default().insert(to);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_include_graph_add() {
        let mut g = IncludeGraph::new();
        g.add_include("src/main.cpp".into(), "include/foo.h".into());
        assert!(g.edges.get(&PathBuf::from("src/main.cpp")).unwrap().contains(&PathBuf::from("include/foo.h")));
    }
}
