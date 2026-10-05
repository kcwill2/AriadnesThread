    use super::{
        collateral_file_percent, stage_end_percent, stage_progress_callback, stage_report,
        stage_start_percent, step_percent, tu_index_percent, ARTIFACTS_PHASE,
        ARTIFACTS_SAVED_PERCENT, CALL_GRAPH_PHASE, COLLATERAL_FILE_SPAN_END, COLLATERAL_PHASE,
        JUSTIFICATION_COMPLETE_PERCENT, JUSTIFYING_SYMBOLS_PERCENT, PAYLOAD_ASSEMBLED_PERCENT,
        PLANNING_EXTRACTIONS_PERCENT, ProgressFn, ProgressReport, REDUCTION_PHASE, STAGES_PER_RUN,
        SYMBOLS_ANALYZED_PERCENT, TU_INDEX_PHASE,
    };
    use ariadne_pipeline::{LoadMilestone, ProgressEvent};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    const POINTS_PER_STAGE: usize = 100 / STAGES_PER_RUN;

    /// The ordered static milestones exactly as `build_ui_payload` emits them,
    /// built from the same constants and helpers the real code path calls.
    /// A4: stage 5's milestone set is {80 start, 81 "Symbols analyzed",
    /// [81,82] file-event span, 85 artifacts-saved / "Collateral mapped" /
    /// "Planning extractions", 86 "Justifying symbols...", 99 "Symbols
    /// justified", 100 payload finalization}, all within [80, 100].
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
            (SYMBOLS_ANALYZED_PERCENT, COLLATERAL_PHASE),
            (ARTIFACTS_SAVED_PERCENT, COLLATERAL_PHASE),
            (PLANNING_EXTRACTIONS_PERCENT, COLLATERAL_PHASE),
            (JUSTIFYING_SYMBOLS_PERCENT, COLLATERAL_PHASE),
            (JUSTIFICATION_COMPLETE_PERCENT, COLLATERAL_PHASE),
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
        assert_eq!(tu_index_percent(0, 0), 20, "empty TU set must sit at stage start");
        assert_eq!(tu_index_percent(0, 7), 20);
        assert_eq!(tu_index_percent(7, 7), 40);
        let mut previous = tu_index_percent(0, 9);
        for done in 1..=9 {
            let percent = tu_index_percent(done, 9);
            assert!((20..=40).contains(&percent), "{done}/9 mapped to {percent}%");
            assert!(percent >= previous, "percent regressed at {done}/9");
            previous = percent;
        }
    }

    #[test]
    fn test_collateral_events_stay_in_stage_five_span() {
        assert_eq!(
            collateral_file_percent(0, 0),
            81,
            "empty collateral set must sit at the file-loop floor"
        );
        assert_eq!(collateral_file_percent(0, 7), 81);
        assert_eq!(collateral_file_percent(7, 7), COLLATERAL_FILE_SPAN_END);
        let mut previous = collateral_file_percent(0, 9);
        for done in 1..=9 {
            let percent = collateral_file_percent(done, 9);
            assert!(
                (SYMBOLS_ANALYZED_PERCENT..=COLLATERAL_FILE_SPAN_END).contains(&percent),
                "{done}/9 mapped to {percent}%"
            );
            assert!(percent >= previous, "percent regressed at {done}/9");
            previous = percent;
        }
    }

    /// A4: replay the exact stage-5 emission order of `run_pipeline_stages`
    /// and `stage_progress_callback` and `build_ui_payload` (static
    /// milestones through the same `stage_report` helper, pipeline events
    /// through the real callback mapping) and pin the produced report
    /// sequence: percent monotonic non-decreasing, every report inside
    /// [80, 100], and exactly one 100% frame — the terminal "Payload
    /// assembled" (progress.rs finalizes only a step-5 100% frame, so this
    /// is the contract that keeps the finalize at the true end of the
    /// build).
    #[test]
    fn test_stage_five_milestone_sequence_is_monotonic_and_contained() {
        let collected: Arc<Mutex<Vec<ProgressReport>>> = Arc::new(Mutex::new(Vec::new()));
        let sink = collected.clone();
        let progress: ProgressFn = Arc::new(move |report| sink.lock().unwrap().push(report));
        let mut event_cb = stage_progress_callback(progress.clone());

        progress(stage_report(5, stage_start_percent(5), COLLATERAL_PHASE, "Load analysis"));
        event_cb(&ProgressEvent::LoadMilestone(LoadMilestone::SymbolsAnalyzed {
            symbols: 21,
        }));
        for done in 0..=4 {
            event_cb(&ProgressEvent::CollateralFile { done, total: 4 });
        }
        event_cb(&ProgressEvent::LoadMilestone(LoadMilestone::ArtifactsSaved));
        progress(stage_report(
            5,
            ARTIFACTS_SAVED_PERCENT,
            COLLATERAL_PHASE,
            "Collateral mapped",
        ));
        progress(stage_report(
            5,
            PLANNING_EXTRACTIONS_PERCENT,
            COLLATERAL_PHASE,
            "Planning extractions",
        ));
        progress(stage_report(
            5,
            JUSTIFYING_SYMBOLS_PERCENT,
            COLLATERAL_PHASE,
            "Justifying symbols...",
        ));
        progress(stage_report(
            5,
            JUSTIFICATION_COMPLETE_PERCENT,
            COLLATERAL_PHASE,
            "Symbols justified",
        ));
        progress(stage_report(
            5,
            PAYLOAD_ASSEMBLED_PERCENT,
            COLLATERAL_PHASE,
            "Payload assembled",
        ));

        let reports = collected.lock().unwrap();
        let sequence: Vec<(usize, &str)> = reports
            .iter()
            .map(|report| (report.percent, report.message.as_str()))
            .collect();
        assert_eq!(
            sequence,
            vec![
                (80, "Load analysis"),
                (81, "21 symbols analyzed"),
                (81, "0/4 files"),
                (81, "1/4 files"),
                (81, "2/4 files"),
                (81, "3/4 files"),
                (82, "4/4 files"),
                (85, "Load artifacts saved"),
                (85, "Collateral mapped"),
                (85, "Planning extractions"),
                (86, "Justifying symbols..."),
                (99, "Symbols justified"),
                (100, "Payload assembled"),
            ],
            "stage-5 emission order must be the calibrated milestone sequence"
        );
        for pair in reports.windows(2) {
            assert!(
                pair[0].percent <= pair[1].percent,
                "percent regressed: {} -> {}",
                pair[0].percent,
                pair[1].percent
            );
        }
        for report in reports.iter() {
            assert_eq!(report.stage, 5, "every stage-5 report stays on step 5");
            assert!(
                (80..=100).contains(&report.percent),
                "{}% escapes the stage-5 span",
                report.percent
            );
        }
        let terminals = reports.iter().filter(|r| r.percent == 100).count();
        assert_eq!(
            terminals, 1,
            "only the terminal 'Payload assembled' may show 100%"
        );
        assert_eq!(reports.last().unwrap().message, "Payload assembled");
    }

    /// A4 minor: the replay above proves the helper mapping is right, but a
    /// future edit could still reorder the real `stage5_report` calls in
    /// `build_ui_payload` and the replay would keep passing. Pin the real
    /// call order in the SOURCE TEXT: inside the production section of
    /// ui.rs (everything before the first test attribute), the byte offset
    /// of "Planning extractions" must be strictly before "Justifying
    /// symbols", and both strictly before the guarded
    /// `build_file_justification_with_memo(` call. Reordering the planner
    /// behind the memo — misrepresenting the boot bar — fails here loudly.
    #[test]
    fn test_planning_milestone_precedes_justifying_in_source() {
        let ui_code = include_str!("../ui.rs");
        let production_end = ui_code
            .find("#[cfg(test)]")
            .expect("ui.rs must have a test module to cut the production section at");
        let production = &ui_code[..production_end];
        // The production cut already excludes this test's own literals.
        let planning_at = production
            .find("Planning extractions")
            .expect("planning milestone label must be emitted in production code");
        let justifying_at = production
            .find("Justifying symbols")
            .expect("justifying milestone label must be emitted in production code");
        // Needle carries the open paren: the doc comments naming the memo
        // function never match, only the real call site does.
        let justification_call_at = production
            .find("build_file_justification_with_memo(")
            .expect("the guarded justification call must exist in production code");
        assert!(
            planning_at < justifying_at,
            "\"Planning extractions\" (byte {planning_at}) must precede \"Justifying symbols\" (byte {justifying_at}) in ui.rs source — the planner really runs before the memo"
        );
        assert!(
            planning_at < justification_call_at,
            "\"Planning extractions\" (byte {planning_at}) must precede the justification call (byte {justification_call_at})"
        );
        assert!(
            justifying_at < justification_call_at,
            "\"Justifying symbols\" (byte {justifying_at}) must immediately precede the call it guards (byte {justification_call_at})"
        );
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
        // The payload builders are inline in `ui.rs` on this branch, so scanning
        // that one file covers every `[k/5]` literal the payload can emit.
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

    /// V3 (I9, handoff test 4): the `BootMilestone` mapping stays inside the
    /// stage-1 span. Mirror of `test_tu_index_events_stay_in_stage_two_span`:
    /// mapped percents sit in-span, `IncludeScan` is monotone over `done`,
    /// and `DiscoveryHashed { total: None }` holds the stage-start percent
    /// honestly (only the message advances). The stage-end frame `(1, 20)`
    /// remains owned by the `run_pipeline_stages` frame (I9).
    #[test]
    fn test_discovery_and_scan_events_stay_in_stage_one_span() {
        use super::{discovery_hashed_percent, include_scan_percent};
        use ariadne_pipeline::{BootMilestone, ProgressEvent};

        assert_eq!(
            discovery_hashed_percent(0, None),
            stage_start_percent(1),
            "message-only hashing (total None) holds the stage start"
        );
        assert_eq!(discovery_hashed_percent(7, None), stage_start_percent(1));
        assert_eq!(discovery_hashed_percent(0, Some(9)), stage_start_percent(1));
        assert_eq!(
            discovery_hashed_percent(9, Some(9)),
            10,
            "bounded hashing ends at the D split point [0, 10]"
        );
        assert_eq!(include_scan_percent(0, 9), 10, "scan starts at the D split");
        assert_eq!(
            include_scan_percent(9, 9),
            19,
            "scan ends one point short of where the stage-end frame owns 20"
        );
        let mut previous = include_scan_percent(0, 9);
        for done in 1..=9 {
            let percent = include_scan_percent(done, 9);
            assert!(
                (10..=19).contains(&percent),
                "{done}/9 scan mapped to {percent}%"
            );
            assert!(percent >= previous, "scan percent regressed at {done}/9");
            previous = percent;
        }

        let collected: Arc<Mutex<Vec<ProgressReport>>> = Arc::new(Mutex::new(Vec::new()));
        let sink = collected.clone();
        let progress: ProgressFn = Arc::new(move |report| sink.lock().unwrap().push(report));
        let mut event_cb = stage_progress_callback(progress.clone());

        progress(stage_report(
            1,
            stage_start_percent(1),
            ARTIFACTS_PHASE,
            "Hashing sources",
        ));
        for done in 1..=3 {
            event_cb(&ProgressEvent::BootMilestone(BootMilestone::DiscoveryHashed {
                done,
                total: None,
            }));
        }
        for done in 1..=4 {
            event_cb(&ProgressEvent::BootMilestone(BootMilestone::IncludeScan {
                done,
                total: 4,
            }));
        }
        progress(stage_report(
            1,
            stage_end_percent(1),
            ARTIFACTS_PHASE,
            "Include graph ready",
        ));

        let reports = collected.lock().unwrap();
        assert_eq!(
            reports[0].percent,
            stage_start_percent(1),
            "the frame-1 first statement stays intact (V1 lock)"
        );
        let event_reports = &reports[1..reports.len() - 1];
        let mut previous = stage_start_percent(1);
        for report in event_reports {
            assert_eq!(report.stage, 1, "boot events stay on step [1/5]");
            assert_eq!(report.phase, ARTIFACTS_PHASE);
            assert!(
                report.percent <= 19,
                "I9: event mapping stops at 19, only the stage-end frame shows 20, got {}",
                report.percent
            );
            assert!(
                report.percent >= previous,
                "stage-1 percent regressed at message {:?}",
                report.message
            );
            previous = report.percent;
        }
        assert_eq!(
            reports[1].percent,
            stage_start_percent(1),
            "total: None hashing holds the stage start through the real callback"
        );
        assert_eq!(
            reports.last().unwrap().percent,
            stage_end_percent(1),
            "the stage-end frame (1, 20) is still produced and closes the stage"
        );
    }

    /// V3 (I10, handoff test 5): the stage-2 Active message must name the
    /// TU's repo-relative path verbatim - the path-in-row that names the
    /// file the next incident dies in. The literal lock includes the double
    /// space before the path (shape sec. 3.1 verbatim literal).
    #[test]
    fn test_tu_started_message_names_verbatim_relative_path() {
        use ariadne_pipeline::ProgressEvent;
        use std::path::PathBuf;

        let collected: Arc<Mutex<Vec<ProgressReport>>> = Arc::new(Mutex::new(Vec::new()));
        let sink = collected.clone();
        let progress: ProgressFn = Arc::new(move |report| sink.lock().unwrap().push(report));
        let mut event_cb = stage_progress_callback(progress);

        event_cb(&ProgressEvent::TuStarted {
            done: 3,
            total: 9,
            source: PathBuf::from("src/some TU.cpp"),
        });

        let reports = collected.lock().unwrap();
        assert_eq!(reports.len(), 1, "one TuStarted maps to exactly one report");
        let report = &reports[0];
        assert_eq!(report.stage, 2);
        assert_eq!(report.phase, TU_INDEX_PHASE);
        assert_eq!(
            report.message, "3/9 TUs  src/some TU.cpp",
            "the message must carry the double space and the verbatim relative path"
        );
        assert_eq!(report.percent, tu_index_percent(3, 9));
    }