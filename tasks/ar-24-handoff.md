# AR-24 Handoff - Typed errors at the extensibility seam (A7)

Wave 6. Crates: lang-api + lang + plugin crates (lang-cpp, lang-scala) —
MULTI-CRATE INTENTIONAL (trait-error change is indivisible across impls).
Depends: AR-18 approved (same crates — serialize: no other lang-lane hand
may be in flight when you start).

## Objective

Replace `Result<_, String>` error traffic at the plugin seam with a typed
thiserror enum in `lang-api` (or `anyhow::Result` where structure adds
nothing — decide per method, journal the table). Convert
`crates/lang/src/validator.rs` `Result<(), String>` and every trait method
returning `String`; adapt lang-cpp/lang-scala impls. MESSAGE TEXT STAYS
STABLE: Display renders the identical strings CLI users see (locked).

## Touch-set (hard)

- `crates/lang-api/src/**` (new error type; trait signature changes)
- `crates/lang/src/validator.rs` (+ any `Result<_, String>` in lang)
- `crates/lang-cpp/src/**` impl adaptations (message-verbatim)
- `crates/lang-scala/src/**` impl adaptations
- `crates/cli/src/registry.rs` — DO NOT re-wrap yet (AR-25 owns dropping the
  `.map_err(anyhow::Error::msg)` re-wraps); only minimal compile adaptations
  if a call site breaks — journal any such touch as a declared rider
- per-crate tests + a new `crates/lang-api/tests/` or inline locks: error-kind
  branching test (match on variant, not string)
- `tasks/ar-24-handoff-journal.md`

## Constraints

- TDD: string-message lock FIRST (capture current exact error strings from
  the impls on representative failures; the typed conversion must reproduce
  each EXACTLY — this is the CLI-visible contract).
- STRICT COMPILE-GREEN across the multi-crate switch (one commit keeps
  everything compiling; red phases = assertion-red only).
- thiserror: check workspace Cargo.toml for availability; if absent, hand-roll
  Display/Error impls (journal; do NOT add new workspace deps without stating
  it as a rider).
- m24 identity / m18 parity suites UNEDITED — their wording locks are the
  external proof messages didn't shift.
- Pure ASCII; commit_scoped per-crate scopes + journal in one commit.

## Gate commands

1. `-p ariadne-lang-api`, `-p ariadne-lang`, `-p ariadne-lang-cpp`,
   `-p ariadne-lang-scala` (each baseline known: 153 cpp, 221 scala —
   reconcile in journal)
2. m18 parity (23/0/0/1) + m24 identity (8/0/0/1) UNEDITED — the CLI-visible
   string authority
3. `-p ariadne-cli` compile + suite (adaptations if any)
4. `encoding_guard --all`; `hygiene_check.py` (only server.rs rows)

## Invariants / locks

- Every pre-conversion error STRING reproduced byte-identically through
  Display (lock list in journal).
- Call sites gain kind-matching ability (demonstrated by the new test)
  without yet being refactored (AR-25 consumes).
