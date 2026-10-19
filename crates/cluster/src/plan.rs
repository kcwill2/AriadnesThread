use ariadne_index::types::SymbolId;
use ariadne_lang::LanguageId;
use ariadne_repo::FileId;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

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
    /// Aggregation file the batch member was found in (plan M23, D-M23-3).
    /// Additive provenance: the isolation engine reads this file's text and
    /// asks the language plugin to locate the symbol's verbatim body.
    /// `#[serde(default)]` keeps older plan artifacts loadable (I6).
    #[serde(default)]
    pub source_file: Option<FileId>,
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
        format!(
            "{}: {} -> {} symbols",
            self.id, self.collateral_load_before, self.collateral_load_after
        )
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
        let _ = writeln!(&mut out, "{}", self.id);
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "Target:");
        let _ = writeln!(&mut out, "    {}", self.target_file.display());
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "Problem:");
        let _ = writeln!(&mut out, "    {}", self.problem);

        if !self.proposed_extractions.is_empty() {
            let _ = writeln!(&mut out);
            let _ = writeln!(&mut out, "Proposed extraction:");
            for ext in &self.proposed_extractions {
                let _ = writeln!(
                    &mut out,
                    "    {} → {}",
                    ext.symbol_name,
                    ext.target_header.display()
                );
            }
        }

        if !self.optimization_frontier.is_empty() {
            let _ = writeln!(&mut out);
            let _ = writeln!(&mut out, "Optimization frontier:");
            for front in &self.optimization_frontier {
                let _ = writeln!(&mut out, "    {}", front.display());
            }
        }

        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "Expected result:");
        let _ = writeln!(
            &mut out,
            "    {} → {} collateral symbols",
            format_number(self.collateral_load_before),
            format_number(self.collateral_load_after)
        );

        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "Risk:");
        let _ = writeln!(&mut out, "    {}", self.risk);

        if !self.validation_required_tus.is_empty() {
            let _ = writeln!(&mut out);
            let _ = writeln!(&mut out, "Validation required:");
            let _ = write!(
                &mut out,
                "    {} translation units",
                self.validation_required_tus.len()
            );
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
    /// The repository contains a language whose plugin reports
    /// `capabilities().planning == false` (plan M22, D23): the language
    /// arrives as neutral `LanguageId` data supplied by the caller, so the
    /// engine stays language-neutral (rule R2).
    PlanningUnsupported(LanguageId),
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
                    source_file: None,
                },
                ProposedExtraction {
                    symbol_name: "ThingBuilder".to_string(),
                    symbol_id: SymbolId::new("sym_11"),
                    target_header: PathBuf::from("thing_builder.h"),
                    source_file: None,
                },
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

    #[test]
    fn planning_unsupported_reason_round_trips_as_language_data() {
        let reason = RejectionReason::PlanningUnsupported(LanguageId::Other);
        let json = serde_json::to_string(&reason).unwrap();
        assert_eq!(
            serde_json::from_str::<RejectionReason>(&json).unwrap(),
            reason
        );
    }

    #[test]
    fn source_file_field_is_additive_for_pre_m23_plan_json() {
        // I6: pre-M23 plan artifacts (no `source_file` key) still load.
        let legacy = r#"{"symbol_name":"A","symbol_id":"A","target_header":"a.h"}"#;
        let parsed: ProposedExtraction = serde_json::from_str(legacy).unwrap();
        assert_eq!(parsed.source_file, None);
    }

    #[test]
    fn pre_existing_rejection_reasons_serialize_unchanged() {
        // The additive variant must not disturb the serialization of the
        // pre-existing ones (variant names and field order).
        assert_eq!(
            serde_json::to_string(&RejectionReason::CycleCreation("x".to_string())).unwrap(),
            r#"{"CycleCreation":"x"}"#
        );
        assert_eq!(
            serde_json::to_string(&RejectionReason::FrozenFile(PathBuf::from("f.h"))).unwrap(),
            r#"{"FrozenFile":"f.h"}"#
        );
        assert_eq!(
            serde_json::to_string(&RejectionReason::ContractViolation(PathBuf::from("c.h")))
                .unwrap(),
            r#"{"ContractViolation":"c.h"}"#
        );
        assert_eq!(
            serde_json::to_string(&RejectionReason::NonMovableDeclaration("y".to_string()))
                .unwrap(),
            r#"{"NonMovableDeclaration":"y"}"#
        );
        assert_eq!(
            serde_json::to_string(&RejectionReason::UnresolvedSemanticDependency(
                "sym_7".to_string()
            ))
            .unwrap(),
            r#"{"UnresolvedSemanticDependency":"sym_7"}"#
        );
    }
}

/// UTA2-R4 item 4 (docs/specs/unit-test-audit.md Round 2): one-shape lock
/// for `format_summary` - the compact id/counts line is THE summary shape.
#[cfg(test)]
mod format_summary_uta2_r4_pin {
    use super::*;

    fn plan_named(id: &str, before: usize, after: usize) -> OptimizationPlan {
        OptimizationPlan {
            id: PlanId(id.to_string()),
            target_file: PathBuf::from("thing.h"),
            target_file_id: FileId::from_relative_path("thing.h"),
            problem: "unused by the summary".to_string(),
            collateral_load_before: before,
            collateral_load_after: after,
            proposed_extractions: vec![],
            optimization_frontier: vec![],
            risk: RiskLevel::Moderate,
            validation_required_tus: vec![],
            rationale: "unused by the summary".to_string(),
        }
    }

    #[test]
    fn pin_format_summary_single_shape() {
        // The one shape: `{id}: {before} -> {after} symbols` - ASCII arrow
        // and NO thousands separators, unlike format_detail (comma groups
        // and its own non-ASCII arrow, both locked by the round-1 test).
        assert_eq!(
            plan_named("PLAN-042", 4382, 1140).format_summary(),
            "PLAN-042: 4382 -> 1140 symbols"
        );
        // Boundary: zero counts keep the same shape.
        assert_eq!(
            plan_named("PLAN-Z", 0, 0).format_summary(),
            "PLAN-Z: 0 -> 0 symbols"
        );
        // Single-line lock: no newline ever enters the summary.
        assert!(!plan_named("PLAN-042", 1_250_000, 7).format_summary().contains('\n'));

        // The summary keys on id + the two counts ONLY: two plans differing
        // in every other field must summarize identically.
        let mut twin = plan_named("PLAN-042", 4382, 1140);
        twin.problem = "a completely different problem".to_string();
        twin.risk = RiskLevel::Aggressive;
        twin.target_file = PathBuf::from("elsewhere.h");
        twin.rationale = "different rationale".to_string();
        twin.proposed_extractions = vec![ProposedExtraction {
            symbol_name: "Any".to_string(),
            symbol_id: SymbolId::new("sym_any"),
            target_header: PathBuf::from("any.h"),
            source_file: None,
        }];
        assert_eq!(twin.format_summary(), plan_named("PLAN-042", 4382, 1140).format_summary());
    }
}
