use std::fmt;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use ariadne_index::types::SymbolId;
use ariadne_repo::FileId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanId(pub String);

impl fmt::Display for PlanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    Conservative,
    Moderate,
    Aggressive,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskLevel::Conservative => write!(f, "Conservative"),
            RiskLevel::Moderate => write!(f, "Moderate"),
            RiskLevel::Aggressive => write!(f, "Aggressive"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProposedExtraction {
    pub symbol_name: String,
    pub symbol_id: SymbolId,
    pub target_header: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OptimizationPlan {
    pub id: PlanId,
    pub target_file: PathBuf,
    pub target_file_id: FileId,
    pub problem: String,
    pub collateral_load_before: usize,
    pub collateral_load_after: usize,
    pub proposed_extractions: Vec<ProposedExtraction>,
    pub optimization_frontier: Vec<PathBuf>,
    pub risk: RiskLevel,
    pub validation_required_tus: Vec<PathBuf>,
    pub rationale: String,
}

impl OptimizationPlan {
    pub fn format_summary(&self) -> String {
        format!("{}: {} -> {} symbols", self.id, self.collateral_load_before, self.collateral_load_after)
    }

    pub fn format_detail(&self) -> String {
        use std::fmt::Write;

        fn format_number(n: usize) -> String {
            let s = n.to_string();
            let mut result = String::new();
            for (i, c) in s.chars().rev().enumerate() {
                if i > 0 && i % 3 == 0 {
                    result.push(',');
                }
                result.push(c);
            }
            result.chars().rev().collect()
        }

        let mut out = String::new();
        writeln!(&mut out, "{}", self.id).unwrap();
        writeln!(&mut out).unwrap();
        writeln!(&mut out, "Target:").unwrap();
        writeln!(&mut out, "    {}", self.target_file.display()).unwrap();
        writeln!(&mut out).unwrap();
        writeln!(&mut out, "Problem:").unwrap();
        writeln!(&mut out, "    {}", self.problem).unwrap();
        
        if !self.proposed_extractions.is_empty() {
            writeln!(&mut out).unwrap();
            writeln!(&mut out, "Proposed extraction:").unwrap();
            for ext in &self.proposed_extractions {
                writeln!(&mut out, "    {} → {}", ext.symbol_name, ext.target_header.display()).unwrap();
            }
        }
        
        if !self.optimization_frontier.is_empty() {
            writeln!(&mut out).unwrap();
            writeln!(&mut out, "Optimization frontier:").unwrap();
            for front in &self.optimization_frontier {
                writeln!(&mut out, "    {}", front.display()).unwrap();
            }
        }
        
        writeln!(&mut out).unwrap();
        writeln!(&mut out, "Expected result:").unwrap();
        writeln!(&mut out, "    {} → {} collateral symbols", format_number(self.collateral_load_before), format_number(self.collateral_load_after)).unwrap();
        
        writeln!(&mut out).unwrap();
        writeln!(&mut out, "Risk:").unwrap();
        writeln!(&mut out, "    {}", self.risk).unwrap();
        
        if !self.validation_required_tus.is_empty() {
            writeln!(&mut out).unwrap();
            writeln!(&mut out, "Validation required:").unwrap();
            write!(&mut out, "    {} translation units", self.validation_required_tus.len()).unwrap();
        }

        out
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RejectionReason {
    CycleCreation(String),
    FrozenFile(PathBuf),
    ContractViolation(PathBuf),
    NonMovableDeclaration(String),
    UnresolvedSemanticDependency(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanRejection {
    pub target_file: PathBuf,
    pub reason: RejectionReason,
    pub explanation: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detail() {
        let plan = OptimizationPlan {
            id: PlanId("PLAN-001".to_string()),
            target_file: PathBuf::from("thing.h"),
            target_file_id: FileId::from_relative_path("thing.h"),
            problem: "4,382 symbols of collateral load".to_string(),
            collateral_load_before: 4382,
            collateral_load_after: 1140,
            proposed_extractions: vec![
                ProposedExtraction {
                    symbol_name: "ThingOptions".to_string(),
                    symbol_id: SymbolId::new("sym_10"),
                    target_header: PathBuf::from("thing_types.h"),
                },
                ProposedExtraction {
                    symbol_name: "ThingBuilder".to_string(),
                    symbol_id: SymbolId::new("sym_11"),
                    target_header: PathBuf::from("thing_builder.h"),
                }
            ],
            optimization_frontier: vec![PathBuf::from("graphics.h")],
            risk: RiskLevel::Conservative,
            validation_required_tus: vec![PathBuf::from("a"); 23],
            rationale: "Some rationale".to_string(),
        };

        let detail = plan.format_detail();
        let expected = "\
PLAN-001

Target:
    thing.h

Problem:
    4,382 symbols of collateral load

Proposed extraction:
    ThingOptions → thing_types.h
    ThingBuilder → thing_builder.h

Optimization frontier:
    graphics.h

Expected result:
    4,382 → 1,140 collateral symbols

Risk:
    Conservative

Validation required:
    23 translation units";

        assert_eq!(detail.trim(), expected.trim());
    }
}
