# Handoff C journal

## C1 (deferred O1: ANSI-strip at pinned parser entry)
- Red: colorized PASS + colorized FAILED fixtures tripped drift at base (2 named failures); colorized DRIFT guard green at base as intended.
- Green: parse_cargo_output now matches over distilled.plain_lines (single-owner strip reused; no regex duplication, distilled.py untouched); Ran 29 OK, discover Ran 183 OK, hygiene exit 0; byte-identity lock untouched.

## C2 (B-gate exit-pin test)
- Green at base confirmed: direct pin (distill + missing --baseline -> header `(exit 2)` AND rc 2 together, tmp-path replay) passed first run with no production change; Ran 30 OK, discover Ran 184 OK, hygiene exit 0.
