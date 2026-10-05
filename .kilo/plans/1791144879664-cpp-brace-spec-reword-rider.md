# cpp-brace-depth-underflow - spec reword rider (M25 closeout)

Why: the M25 closeout ledger flip states the shipped G4 acceptance axis. Spec
`docs/specs/cpp-brace-depth-underflow.md` still carries wording the reviewer
adjudicated unsatisfiable by layer 1 ("POST-fix zero TUs below zero",
"corpus scan output diff empty on the balanced set") plus stale parity counts
(m18 "22/22" - the UNEDITED file holds 23 tests; m17 "5/5" - holds 6). The
closeout session (planner) has an edit ceiling that denies `docs/specs/`
(documented wall, ledger deviation note + deferred finding row); apply these
edits from a session that holds edit there. Evidence:
`tasks/cpp-brace-depth-underflow-execution-results.md` round-2 G4 section and
`tasks/cpp-brace-depth-underflow-t1-verdict.md` (round-2 addendum, finding 3 /
D7). Pre-reword spec bytes are preserved in blob `9ce140a`.

All replacements are exact oldString -> newString pairs; content pure ASCII.

## E1 - status header

Old:
```
Status: PLANNED (architect artifact; planner sizes handoffs from this doc).
Milestone key `cpp-brace-depth-underflow`; M-number and dates are assigned by
milestone-lifecycle kickoff - do not guess them.
```
New:
```
Status: CLOSED 2026-10-04 as M25 (number assigned at closeout - no kickoff doc
existed; M15-M24 and M23.5 already consumed). Shipped on `main` through
`af11113`; verdict `tasks/cpp-brace-depth-underflow-t1-verdict.md` round 2
APPROVED. Closeout reword: G4(b), I3, and 4.3 ("zero TUs below zero" /
"balanced-set digest diff empty") proved unsatisfiable by layer 1 - the scan
replays raw token arithmetic and the fix deliberately changes raw-hazard
token emissions - and were reworded to the shipped acceptance axis (see those
sections; evidence in `tasks/cpp-brace-depth-underflow-execution-results.md`
round-2 G4). Parity counts corrected at closeout: m18 22->23, m17 5->6 (the
UNEDITED files always held 23 and 6 tests).
```

## E2 - invariant I3

Old:
```
I3 Balanced-stream freeze. For every input whose emitted token stream is
    prefix-nonnegative and closes at zero (all current tests, all
    `tests/fixtures/**`, and all corpus TUs the scan classifies balanced),
    symbols/declarations/edges/call_sites are byte-identical to the pre-fix
    implementation. Oracles: characterization suites green UNEDITED,
    `m18_cpp_plugin_parity` 22/22 UNEDITED, `m00_golden_baseline` zero drift,
    corpus scan output diff empty on the balanced set.
```
New:
```
I3 Balanced-stream freeze (reworded at M25 closeout - verdict finding 3/D7;
    the original absolute form was unsatisfiable by layer 1). For every input
    whose emitted token stream is prefix-nonnegative, closes at zero, and
    contains no raw-string literal (all current tests, all
    `tests/fixtures/**` - grep-verified raw-free - and the non-raw balanced
    corpus TUs), symbols/declarations/edges/call_sites are byte-identical to
    the pre-fix implementation. Raw-hazard TUs are layer 1's intended change
    class (3.2): pre-fix quote flips emitted phantom brace tokens whose
    removal is the point of the fix; for those TUs the freeze holds at
    depth-state level (pre == post state). Oracles: characterization suites
    green UNEDITED, `m18_cpp_plugin_parity` 23/23 UNEDITED (wording drift:
    file holds 23 tests), `m00_golden_baseline` zero drift, corpus scan with
    zero committed-vs-faithful digest/state diffs, zero newly-negative TUs,
    and every balanced-set-vs-PRE digest diff attributable to a raw-hazard TU
    with unchanged depth state (shipped: 17 such TUs, zero state changes).
```

## E3 - lock table rows (3.4)

Old:
```
| `m18_cpp_plugin_parity` 22/22, UNEDITED | `python tools/testrun.py --distill -p ariadne-integration-tests --test m18_cpp_plugin_parity_test` |
| `m17_plugin_seam_test` 5/5 (architecture guard) | `python tools/testrun.py --distill -p ariadne-integration-tests --test m17_plugin_seam_test` |
```
New:
```
| `m18_cpp_plugin_parity` 23/23, UNEDITED (closeout correction: wording drift, file holds 23 tests) | `python tools/testrun.py --distill -p ariadne-integration-tests --test m18_cpp_plugin_parity_test` |
| `m17_plugin_seam_test` 6/6 (architecture guard; closeout correction: file holds 6 tests) | `python tools/testrun.py --distill -p ariadne-integration-tests --test m17_plugin_seam_test` |
```

## E4 - gate G2 (3.5)

Old:
```
- G2 `python tools/testrun.py --distill -p ariadne-integration-tests --test m18_cpp_plugin_parity_test` - 22/22.
```
New:
```
- G2 `python tools/testrun.py --distill -p ariadne-integration-tests --test m18_cpp_plugin_parity_test` - 23/23 (closeout correction: wording drift; round-2 line `23 passed / 0 failed / 0 ignored / 1 suites`).
```

## E5 - gate G4(b) (3.5)

Old:
```
  line of first underflow; (b) POST-fix: zero TUs go below zero. The reviewer
  records the output verbatim; this pins "the 336th TU" to a named file:line.
```
New:
```
  line of first underflow; (b) POST-fix acceptance axis (reworded at M25
  closeout - the literal "zero TUs go below zero" is unsatisfiable by layer 1:
  the scan replays raw token arithmetic and therefore surfaces the spec-2.3
  residual class the layer-2 parser floor exists to absorb): zero
  committed-vs-faithful digest/state diffs, zero newly-negative TUs vs PRE,
  and the below-zero set equal to the spec-2.3 undecidable class (shipped:
  14 residual TUs, faithful mode also 14). The reviewer records the output
  verbatim; (a) pinned the 336th-TU question to
  `AggregateFunctionNull.cpp:174`.
```

## E6 - oracle discipline (4.3)

Old:
```
bytes; (3) G4's throwaway replay scan is the corpus-wide equivalence check on
balanced TUs (run the balanced-set digest before and after; any diff on a
balanced TU is a blocking finding). No new fixture directories under
```
New:
```
bytes; (3) G4's throwaway replay scan is the corpus-wide equivalence check
(reworded at M25 closeout): run the balanced-set digest before and after; a
depth-STATE diff, or a digest diff on a TU with no raw-string header, is a
blocking finding; digest diffs confined to raw-hazard TUs with unchanged depth
state are the intended layer-1 change and must be enumerated with their
raw-header counts (shipped: 17 TUs, raw_headers 1..1375, incl.
`Server\ACME\API.cpp`; zero state changes, zero non-raw diffs). No new fixture directories under
```

## E7 - risk table row (4.4)

Old:
```
| `#if 0`-style residual drift stays wrong-but-defined and is mistaken for a bug | I5 defines the expected degradation and a test pins it; G4 scan reports post-fix zero-negative as the pass condition |
```
New:
```
| `#if 0`-style residual drift stays wrong-but-defined and is mistaken for a bug | I5 defines the expected degradation and a test pins it; G4 pass axis is newly_negative=0 with the residual below-zero set matching the spec-2.3 floor-absorbed class (G4(b) reworded at M25 closeout) |
```

## Rider commit (after applying E1-E7)

```
python tools/encoding_guard.py --staged
python tools/commit_scoped.py --scope "docs/specs/cpp-brace-depth-underflow.md" -m "docs(specs): reword cpp-brace-depth-underflow G4(b)/I3/4.3 to shipped acceptance axis + parity counts (M25 closeout rider, verdict finding 3)"
```

After the rider lands, flip the M25 ledger row clause "reword DELEGATED: edit
ceiling denies `docs/specs/`; text in board post, rider owed" to "reword
LANDED <hash> (rider)".
