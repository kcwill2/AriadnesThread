# AR-26a2 Handoff Journal - atomic manager/* move into ariadne-artifact (A3 chain, hop 2/3)

Contract: `tasks/ar-26a2-handoff.md` (amended 69ab3f7 with machine-checkable
rider gate 7 + current baselines). Binding inputs read in full:
`tasks/ar-26a-verdict.md` F1-F3, `tasks/ar-26a-handoff-journal.md`
"ADJUDICATED EXECUTION" + "RIDER FOR 26a2" (ruling board_12d48349).

## Dispatch HEAD + baselines (re-measured BEFORE mutation, own stdout)

- Dispatch HEAD `69ab3f7` (docs-only contract amendment over hop-1 landing).
- Code anchor `4794467` (hop-1 landing); journal tip `075e3bf` (zero code
  delta). FROZEN-LOCK DIFF BASE: `4794467` (journal the choice: code anchor).
- Foreign at dispatch, untouched: `tests/fixtures/basic/artifacts/includes/
  report.json` LF/CRLF churn (same foreign churn as hop-1 dispatch; left
  unstaged); untracked docs under `tasks/**` + `.kilo/plans/**` (verdict /
  execution-results / handoff docs of prior ARs) - never staged by my scopes.

Suite baselines (all `python tools/testrun.py --distill ...`, own stdout,
capture `target/testrun-capture.txt` per run):

- `-p ariadne-report`: `47 passed / 0 failed / 0 ignored / 4 suites`
- `-p ariadne-artifact`: `0 passed / 0 failed / 0 ignored / 2 suites`
- `-p ariadne-scheduler`: `26 passed / 0 failed / 0 ignored / 2 suites`
- `-p ariadne-cli`: `118 passed / 0 failed / 8 ignored / 7 suites`
- m00 golden pair: `4 passed / 0 failed / 0 ignored / 2 suites` UNEDITED
- m23_symbol_isolation_test: `7 passed / 0 failed / 0 ignored / 1 suites` UNEDITED
- m17_plugin_seam_test: `8 passed / 0 failed / 0 ignored / 1 suites` UNEDITED
- `python tools/hygiene_check.py`: R3 (warn) rows only, ZERO blocking rows
  (post-AR-13 baseline row set; every listed row is R3 warn).
- CENSUS ANCHOR (contract line; reviewer duty per dispatch brief - NOT
  re-run here): `1398 passed / 0 failed / 13 ignored / 105 suites`.

## Pre-mutation inventory verification

- 11 files present: `crates/report/src/manager.rs` + `manager/{apply_io,
  bincode_options, discovery_io, graph_io, index_io, metrics_io,
  transform_io, tests, m22_manifest_languages_tests,
  payload_reshape_tests}.rs`.
- All 9 `use crate::manifests::{...}` sites name `crate::manifests` (NO
  `ariadne_artifact::manifests` drift anywhere -> no contract drift, no
  stop): apply_io.rs:10, discovery_io.rs:12, graph_io.rs:12, index_io.rs:10,
  metrics_io.rs:13, transform_io.rs:10, tests.rs:2,
  m22_manifest_languages_tests.rs:6, payload_reshape_tests.rs:20.
  Three fn-importers (import `collect_stage_languages`): discovery_io,
  metrics_io, graph_io.
- `crate::` REBINDING choice (contract decision note, verbatim): the 9
  import lines stay BYTE-UNCHANGED. At the new home `crate::` rebinds to
  the artifact crate root, where the real `pub mod manifests` provides the
  12 DTOs and the re-privatized `pub(crate)` fn via same-crate visibility.
  The import TEXT does not change; only its resolution target does, from
  the report shim binding to the same-crate module (resolution-time, the
  same E-class the AR-20a shim rule handled).
- Moved-file dep surface (grep-proven): anyhow, serde_json, bincode, rayon,
  ariadne-build, ariadne-index, ariadne-cluster (metrics_io + tests),
  ariadne-transform (tests only), ariadne-validate (apply_io), tempfile
  (3 test mods -> dev-dep). serde/graph/lang/repo already in artifact
  Cargo.toml. Matches the contract's addition list exactly.
- `collect_stage_languages` external-consumer census: zero consumers outside
  the artifact def + the 3 IO importers + the report rider line. Gate 7
  terminal state reachable.
- m17 transitive cone guard walks only plugin crates (lang-cpp/lang-scala);
  artifact is unreachable from plugin manifests, so the new artifact
  cluster/transform/validate edges cannot fire it this hop (cone guard
  update is AR-26b's job, file stays UNEDITED).

## Deviations / findings so far

- None.
