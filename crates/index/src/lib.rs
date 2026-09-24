//! Semantic indexing abstractions.

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SymbolId(pub String);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DependencyKind {
    Complete,
    Incomplete,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_id() {
        let sym = SymbolId("Thing".to_string());
        assert_eq!(sym.0, "Thing");
    }
}
