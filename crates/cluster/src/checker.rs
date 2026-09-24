use ariadne_index::types::SymbolId;
use ariadne_repo::OptimizationPolicy;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::plan::{PlanRejection, RejectionReason};

pub struct ConstraintChecker {
    pub file_policies: HashMap<PathBuf, OptimizationPolicy>,
    pub non_movable_symbols: HashSet<SymbolId>,
}

impl ConstraintChecker {
    pub fn new(
        file_policies: HashMap<PathBuf, OptimizationPolicy>,
        non_movable_symbols: HashSet<SymbolId>,
    ) -> Self {
        Self {
            file_policies,
            non_movable_symbols,
        }
    }

    pub fn validate_extraction(
        &self,
        symbol: &SymbolId,
        target_file: &Path,
        splits_scc: bool,
        creates_include_cycle: bool,
        unresolved_dep: Option<String>,
    ) -> Result<(), PlanRejection> {
        if let Some(policy) = self.file_policies.get(target_file) {
            match policy {
                OptimizationPolicy::Frozen => {
                    return Err(PlanRejection {
                        target_file: target_file.to_path_buf(),
                        reason: RejectionReason::FrozenFile(target_file.to_path_buf()),
                        explanation: format!("File {} is frozen and cannot be modified.", target_file.display()),
                    });
                }
                OptimizationPolicy::Contract => {
                    return Err(PlanRejection {
                        target_file: target_file.to_path_buf(),
                        reason: RejectionReason::ContractViolation(target_file.to_path_buf()),
                        explanation: format!("File {} is a contract API.", target_file.display()),
                    });
                }
                _ => {}
            }
        }

        if self.non_movable_symbols.contains(symbol) {
            return Err(PlanRejection {
                target_file: target_file.to_path_buf(),
                reason: RejectionReason::NonMovableDeclaration(symbol.0.clone()),
                explanation: format!("Symbol {} is non-movable.", symbol.0),
            });
        }

        if let Some(dep) = unresolved_dep {
            return Err(PlanRejection {
                target_file: target_file.to_path_buf(),
                reason: RejectionReason::UnresolvedSemanticDependency(dep.clone()),
                explanation: format!("Unresolved semantic dependency: {}", dep),
            });
        }

        if splits_scc || creates_include_cycle {
            let msg = if splits_scc {
                "Extraction splits a strongly connected component".to_string()
            } else {
                "Extraction introduces a cycle in header includes".to_string()
            };
            return Err(PlanRejection {
                target_file: target_file.to_path_buf(),
                reason: RejectionReason::CycleCreation(msg.clone()),
                explanation: msg,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_checker() -> ConstraintChecker {
        let mut policies = HashMap::new();
        policies.insert(PathBuf::from("frozen.h"), OptimizationPolicy::Frozen);
        policies.insert(PathBuf::from("contract.h"), OptimizationPolicy::Contract);
        policies.insert(PathBuf::from("ok.h"), OptimizationPolicy::Optimizable);

        let mut non_movable = HashSet::new();
        non_movable.insert(SymbolId::new("NonMovable"));

        ConstraintChecker::new(policies, non_movable)
    }

    #[test]
    fn test_valid_extraction() {
        let checker = setup_checker();
        let result = checker.validate_extraction(
            &SymbolId::new("Movable"),
            Path::new("ok.h"),
            false,
            false,
            None,
        );
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn test_rejection_cycle_creation_scc() {
        let checker = setup_checker();
        let result = checker.validate_extraction(
            &SymbolId::new("Movable"),
            Path::new("ok.h"),
            true, // splits_scc
            false,
            None,
        );
        assert!(matches!(
            result,
            Err(PlanRejection { reason: RejectionReason::CycleCreation(_), .. })
        ));
    }

    #[test]
    fn test_rejection_cycle_creation_include() {
        let checker = setup_checker();
        let result = checker.validate_extraction(
            &SymbolId::new("Movable"),
            Path::new("ok.h"),
            false,
            true, // creates_include_cycle
            None,
        );
        assert!(matches!(
            result,
            Err(PlanRejection { reason: RejectionReason::CycleCreation(_), .. })
        ));
    }

    #[test]
    fn test_rejection_frozen_file() {
        let checker = setup_checker();
        let result = checker.validate_extraction(
            &SymbolId::new("Movable"),
            Path::new("frozen.h"),
            false,
            false,
            None,
        );
        assert!(matches!(
            result,
            Err(PlanRejection { reason: RejectionReason::FrozenFile(_), .. })
        ));
    }

    #[test]
    fn test_rejection_contract_violation() {
        let checker = setup_checker();
        let result = checker.validate_extraction(
            &SymbolId::new("Movable"),
            Path::new("contract.h"),
            false,
            false,
            None,
        );
        assert!(matches!(
            result,
            Err(PlanRejection { reason: RejectionReason::ContractViolation(_), .. })
        ));
    }

    #[test]
    fn test_rejection_non_movable() {
        let checker = setup_checker();
        let result = checker.validate_extraction(
            &SymbolId::new("NonMovable"),
            Path::new("ok.h"),
            false,
            false,
            None,
        );
        assert!(matches!(
            result,
            Err(PlanRejection { reason: RejectionReason::NonMovableDeclaration(_), .. })
        ));
    }

    #[test]
    fn test_rejection_unresolved_dep() {
        let checker = setup_checker();
        let result = checker.validate_extraction(
            &SymbolId::new("Movable"),
            Path::new("ok.h"),
            false,
            false,
            Some("MissingDep".to_string()),
        );
        assert!(matches!(
            result,
            Err(PlanRejection { reason: RejectionReason::UnresolvedSemanticDependency(_), .. })
        ));
    }
}
