use super::{
    collateral_file_percent, stage_end_percent, stage_start_percent, step_percent,
    tu_index_percent, ARTIFACTS_PHASE, CALL_GRAPH_PHASE, COLLATERAL_MAPPED_PERCENT,
    COLLATERAL_PHASE, REDUCTION_PHASE, STAGES_PER_RUN, TU_INDEX_PHASE,
};
use std::collections::HashMap;

const POINTS_PER_STAGE: usize = 100 / STAGES_PER_RUN;

/// The ordered static milestones exactly as `build_ui_payload` emits them,
/// built from the same constants and helpers the real code path calls.
/// Stage 5's milestone set is {80 start, 80–95 event span, 95 "Collateral
/// mapped", 100 payload finalization}, all within [80, 100].
fn emitted_milestones() -> Vec<(usize, &'static str)> {
    vec![
        (stage_start_percent(1), ARTIFACTS_PHASE),
        (stage_end_percent(1), ARTIFACTS_PHASE),
        (stage_start_percent(2), TU_INDEX_PHASE),
        (stage_end_percent(2), TU_INDEX_PHASE),
        (stage_start_percent(3), REDUCTION_PHASE),
        (stage_end_percent(3), REDUCTION_PHASE),
        (stage_start_percent(4), CALL_GRAPH_PHASE),
        (stage_end_percent(4), CALL_GRAPH_PHASE),
        (stage_start_percent(5), COLLATERAL_PHASE),
        (COLLATERAL_MAPPED_PERCENT, COLLATERAL_PHASE),
        (100, COLLATERAL_PHASE),
    ]
}

fn stage_number_of(phase: &str) -> usize {
    phase
        .as_bytes()
        .get(1)
        .and_then(|byte| (*byte as char).to_digit(10))
        .unwrap_or_else(|| panic!("milestone phase must carry a [k/5] label: {phase:?}"))
        as usize
}

#[test]
fn test_stage_milestones_match_step_labels() {
    let milestones = emitted_milestones();
    for pair in milestones.windows(2) {
        assert!(
            pair[0].0 <= pair[1].0,
            "percent must be monotonic, got {} before {}",
            pair[0].0,
            pair[1].0
        );
    }
    for (percent, phase) in &milestones {
        let stage = stage_number_of(phase);
        let lower = (stage - 1) * POINTS_PER_STAGE;
        let upper = stage * POINTS_PER_STAGE;
        assert!(
            (lower..=upper).contains(percent),
            "milestone {phase} at {percent}% escapes its [{lower}, {upper}] span"
        );
    }
    // The contract table must not be fiction: the payload-assembly
    // milestone that finalizes step 5 at 100% must really be emitted.
    // Built via format! so this test's own source cannot match itself.
    let assembled_needle = format!("Payload {}", "assembled");
    assert!(
        include_str!("../ui.rs").contains(assembled_needle.as_str()),
        "step 5 must finalize with a 100% payload-assembly milestone"
    );
}

#[test]
fn test_step_percent_maps_stage_span_onto_zero_hundred() {
    assert_eq!(step_percent(1, 0), 0);
    assert_eq!(step_percent(1, 10), 50);
    assert_eq!(step_percent(1, 20), 100);
    assert_eq!(step_percent(2, 20), 0, "stage start is step start");
    assert_eq!(step_percent(2, 30), 50, "stage midpoint is 50% of the step");
    assert_eq!(step_percent(2, 40), 100, "stage end is step end");
    assert_eq!(step_percent(5, 80), 0);
    assert_eq!(step_percent(5, 90), 50);
    assert_eq!(step_percent(5, 100), 100);
    assert_eq!(step_percent(2, 5), 0, "clamped below own span");
    assert_eq!(step_percent(2, 95), 100, "clamped above own span");
    assert_eq!(step_percent(5, 120), 100, "clamped past 100");
}

#[test]
fn test_tu_index_events_stay_in_stage_two_span() {
    assert_eq!(
        tu_index_percent(0, 0),
        20,
        "empty TU set must sit at stage start"
    );
    assert_eq!(tu_index_percent(0, 7), 20);
    assert_eq!(tu_index_percent(7, 7), 40);
    let mut previous = tu_index_percent(0, 9);
    for done in 1..=9 {
        let percent = tu_index_percent(done, 9);
        assert!(
            (20..=40).contains(&percent),
            "{done}/9 mapped to {percent}%"
        );
        assert!(percent >= previous, "percent regressed at {done}/9");
        previous = percent;
    }
}

#[test]
fn test_collateral_events_stay_in_stage_five_span() {
    assert_eq!(
        collateral_file_percent(0, 0),
        80,
        "empty collateral set must sit at stage start"
    );
    assert_eq!(collateral_file_percent(0, 7), 80);
    assert_eq!(collateral_file_percent(7, 7), 95);
    let mut previous = collateral_file_percent(0, 9);
    for done in 1..=9 {
        let percent = collateral_file_percent(done, 9);
        assert!(
            (80..=95).contains(&percent),
            "{done}/9 mapped to {percent}%"
        );
        assert!(percent >= previous, "percent regressed at {done}/9");
        previous = percent;
    }
}

#[test]
fn test_phase_label_stable_within_stage() {
    let mut phase_by_stage: HashMap<usize, &str> = HashMap::new();
    for (_, phase) in emitted_milestones() {
        let stage = stage_number_of(phase);
        if let Some(previous) = phase_by_stage.insert(stage, phase) {
            assert_eq!(
                previous, phase,
                "stage {stage} milestones disagree: {previous:?} vs {phase:?}"
            );
        }
    }
    let ui_code = include_str!("../ui.rs");
    for stage in 1..=STAGES_PER_RUN {
        let prefix = format!("[{stage}/5]");
        let tail = &phase_by_stage[&stage][prefix.len()..];
        let mut haystack = ui_code;
        while let Some(index) = haystack.find(&prefix) {
            let rest = &haystack[index + prefix.len()..];
            assert!(
                rest.starts_with(tail),
                "stage {stage} emits an inconsistent phase label near: {:?}",
                &rest[..rest.len().min(32)]
            );
            haystack = rest;
        }
    }
}
