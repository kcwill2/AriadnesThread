"""Debug pass 3: full missing-line list, order-violation scope, raised-region
line-exact compare, contract byte-append proof. Ephemeral; delete after run."""
import subprocess

def blob(rev_path):
    return subprocess.check_output(["git", "cat-file", "blob", rev_path])

START = "5c81881"
C1 = "1819b8f"
HEAD = "1e81d7e"

old = blob(f"{START}:crates/pipeline/src/lib.rs")
old_lines = old.splitlines(keepends=True)
sl_lines = blob(f"{C1}:crates/pipeline/src/stage_load.rs").splitlines(keepends=True)
lib1_lines = blob(f"{C1}:crates/pipeline/src/lib.rs").splitlines(keepends=True)

# extent of the start-rev import block: last line < 60 that starts with use/#!
last_import = max(i for i, l in enumerate(old_lines[:60], 1)
                  if l.startswith(b"use ") or l.startswith(b"#[") or l.startswith(b")") or l.startswith(b"}"))
print("start-rev import block extends to at least line", last_import)

CHUNKS_INDEX = [(382, 537), (779, 798), (822, 891)]
CHUNKS_LOAD = [(617, 741), (893, 952)]
moved = set()
for a, b in CHUNKS_INDEX + CHUNKS_LOAD:
    moved.update(range(a, b + 1))
kept = [(i, l) for i, l in enumerate(old_lines, 1)
        if i not in moved and l.strip()]
new_set = {}
for j, l in enumerate(lib1_lines):
    new_set.setdefault(l, []).append(j + 1)
misses = [(i, l) for i, l in kept if l not in new_set]
print(f"missing kept lines: {len(misses)}")
for i, l in misses:
    print(f"  MISSING old:{i}: {l[:90]!r}")

survivors = [(i, l) for i, l in kept if l in new_set]
viol = []
last_used = -1
for i, l in survivors:
    nxt = next((p for p in new_set[l] if p > last_used), None)
    if nxt is None:
        viol.append((i, l))
    else:
        last_used = nxt
print(f"order-violating survivors: {len(viol)}")
for i, l in viol[:20]:
    print(f"  VIO old:{i}: {l[:90]!r}")
print("all missing+violating lines have start-rev line numbers:",
      sorted({i for i, _ in misses} | {i for i, _ in viol}))
non_import = [i for i, _ in misses + viol if i > last_import]
print("NON-IMPORT-BLOCK offenders (line >", last_import, "):", non_import)

print("== raised-region line-exact: old 893-952 vs stage_load.rs 151-210 ==")
oa, nb = old_lines[892:952], sl_lines[150:210]
diffs = [(893 + k, oa[k], 151 + k, nb[k]) for k in range(60) if oa[k] != nb[k]]
for o_ln, o_l, n_ln, n_l in diffs:
    print(f"  DIFF old:{o_ln} {o_l!r} vs new:{n_ln} {n_l!r}")
print(f"  total diffs in region: {len(diffs)}")

pct_old = blob(f"{START}:crates/cli/src/commands/ui/progress_contract_tests.rs")
pct_head = blob(f"{HEAD}:crates/cli/src/commands/ui/progress_contract_tests.rs")
print("contract old ends with newline:", pct_old.endswith(b"\n"),
      "| head starts with old bytes:",
      pct_head.startswith(pct_old if pct_old.endswith(b"\n") else pct_old + b"\n"),
      "| appended byte count:",
      len(pct_head) - len(pct_old + (b"" if pct_old.endswith(b"\n") else b"\n")))
