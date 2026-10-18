//! UTA-T1 item 13 (hand H1): depth-stress pins for the closures.rs SCC
//! machinery on a 64 KiB thread (mirroring the IV-7 pattern from
//! `crates/graph/tests/cycle_groups_test.rs:105`).
//!
//! DEVIATION (recorded in tasks/uta-t1-journal.md): the handoff asks for
//! "100k-node cycle compute_closures on a 64 KiB stack". `compute_closures`
//! materializes a full `transitive_includes` Vec PER member, so a fused
//! 100k cycle costs O(V^2) element copies + sorts (~10^11 ops at V=100k -
//! far beyond the 600 s gate regardless of stack). The stress is therefore
//! split: the iterative Tarjan frame machine - the ONLY recursive-overflow
//! risk surface in the module - runs the FULL 100k fused cycle via the
//! crate-internal `tarjan_components` seam (linear work), and the public
//! `compute_closures` pipeline (condensation DP + assembly) runs end-to-end
//! on a 3k fused cycle, deep enough that a recursive DFS would have
//! overflowed the same 64 KiB thread.

use super::*;

const SMALL_STACK_BYTES: usize = 64 * 1024;
const FULL_CYCLE_LEN: u64 = 100_000;
const PIPELINE_CYCLE_LEN: u64 = 3_000;

/// f0 -> f1 -> ... -> f(N-1) -> f0 as sorted-destinations adjacency.
fn fused_cycle_adjacency(len: u64) -> HashMap<FileId, Vec<FileId>> {
    (0..len)
        .map(|position| {
            (
                FileId(position),
                vec![FileId((position + 1) % len)],
            )
        })
        .collect()
}

fn fused_cycle_nodes(len: u64) -> Vec<FileId> {
    (0..len).map(FileId).collect()
}

#[test]
fn fused_100k_cycle_tarjan_frame_machine_completes_on_small_stack() {
    // The frame machine (`tarjan_components`/`explore_frames`) is the
    // deep-overflow risk surface UTA-B5 names for this module; on a 64 KiB
    // thread a recursive DFS dies within a few hundred frames, the
    // explicit-stack machine walks all 100 000.
    let spawned = std::thread::Builder::new()
        .stack_size(SMALL_STACK_BYTES)
        .spawn(run_full_depth_tarjan)
        .expect("deep-cycle stress thread spawns with the requested stack size");
    let (components, component_of) =
        spawned.join().expect("deep-cycle stress thread completed without panicking");
    assert_eq!(
        components.len(),
        1,
        "the fused 100 000-node cycle condenses to exactly one SCC"
    );
    assert_eq!(
        components[0].len() as u64, FULL_CYCLE_LEN,
        "every cycle member is in the single component"
    );
    assert_eq!(component_of.len() as u64, FULL_CYCLE_LEN);
}

fn run_full_depth_tarjan() -> (Vec<Vec<FileId>>, HashMap<FileId, usize>) {
    let adjacency = fused_cycle_adjacency(FULL_CYCLE_LEN);
    let nodes = fused_cycle_nodes(FULL_CYCLE_LEN);
    tarjan_components(&nodes, &adjacency)
}

#[test]
fn fused_cycle_compute_closures_pipeline_completes_on_small_stack() {
    // End-to-end through the PUBLIC entry on a deep fused cycle: Tarjan,
    // condensation DP and closure assembly all run on the 64 KiB thread.
    // 3 000 members already exceed a recursive walk's survivable depth by
    // an order of magnitude while keeping the O(V^2) materialization of
    // the API contract inside the test budget (see module header).
    let spawned = std::thread::Builder::new()
        .stack_size(SMALL_STACK_BYTES)
        .spawn(run_pipeline_on_fused_cycle)
        .expect("pipeline stress thread spawns with the requested stack size");
    let closures = spawned
        .join()
        .expect("pipeline stress thread completed without panicking");
    assert_eq!(
        closures.len() as u64, PIPELINE_CYCLE_LEN,
        "one closure row per cycle member"
    );
    for (&file_id, closure) in &closures {
        assert_eq!(
            closure.closure_size as u64,
            PIPELINE_CYCLE_LEN - 1,
            "every member reaches every other member of the fused cycle"
        );
        assert!(
            !closure.transitive_includes.contains(&file_id),
            "the BFS seed-exclusion convention holds at depth: a member's \
             closure never contains itself"
        );
    }
}

fn run_pipeline_on_fused_cycle() -> HashMap<FileId, crate::types::IncludeClosure> {
    let adjacency = fused_cycle_adjacency(PIPELINE_CYCLE_LEN);
    let all_files: HashSet<FileId> = fused_cycle_nodes(PIPELINE_CYCLE_LEN)
        .into_iter()
        .collect();
    compute_closures(&adjacency, &all_files)
}
