"""Reviewer-independent verbatim-extraction proof for boot-progress V3 commit 1819b8f.

Byte-level via git cat-file (no PowerShell text pipes). Checks:
 1. Each declared moved chunk is a byte-identical contiguous substring of
    BOTH the start-rev lib.rs blob and the landing module blob (one
    sanctioned pub(crate) raise allowed on collect_file_metrics).
 2. All kept non-blank lib.rs lines (outside moved chunks) survive in order
    in the post-move lib.rs.
 3. Line counts at the declared revisions.
 4. stage_load.rs unchanged between 1819b8f and HEAD.
 5. progress_contract_tests.rs append-only: existing prefix byte-identical.
 6. progress.rs byte-identical 5c81881 -> HEAD.

Ephemeral review artifact; delete after running.
"""
import subprocess

def blob(rev_path):
    return subprocess.check_output(["git", "cat-file", "blob", rev_path])

START = "5c81881"
C1 = "1819b8f"
HEAD = "1e81d7e"

old = blob(f"{START}:crates/pipeline/src/lib.rs")
old_lines = old.splitlines(keepends=True)

def chunk(a, b):  # 1-based inclusive
    return b"".join(old_lines[a - 1:b])

CHUNKS_INDEX = [(382, 537), (779, 798), (822, 891)]
CHUNKS_LOAD = [(617, 741), (893, 952)]

si = blob(f"{C1}:crates/pipeline/src/stage_index.rs")
sl = blob(f"{C1}:crates/pipeline/src/stage_load.rs")
sl_head = blob(f"{HEAD}:crates/pipeline/src/stage_load.rs")
lib1 = blob(f"{C1}:crates/pipeline/src/lib.rs")

fails = []

def report(name, ok, detail=""):
    print(("PASS" if ok else "FAIL"), name, detail)
    if not ok:
        fails.append(name)

# 1a. index chunks byte-identical in old lib and in stage_index.rs
for a, b in CHUNKS_INDEX:
    c = chunk(a, b)
    report(f"chunk {a}-{b} in old lib contiguous", old.count(c) == 1,
           f"occurrences={old.count(c)}")
    report(f"chunk {a}-{b} verbatim in stage_index.rs", si.count(c) == 1,
           f"occurrences={si.count(c)}")

# 1b. load chunks
for a, b in CHUNKS_LOAD[:-1]:
    c = chunk(a, b)
    report(f"chunk {a}-{b} in old lib contiguous", old.count(c) == 1)
    report(f"chunk {a}-{b} verbatim in stage_load.rs", sl.count(c) == 1)

a, b = CHUNKS_LOAD[-1]  # collect_file_metrics region: sanctioned raise allowed
c = chunk(a, b)
raised = c.replace(b"    fn collect_file_metrics", b"    pub(crate) fn collect_file_metrics")
report("collect_file_metrics region == old + single pub(crate) raise ONLY",
       sl.count(raised) == 1 and c.count(b"    fn collect_file_metrics") == 1,
       f"old_decl={c.count(b'    fn collect_file_metrics')} raised_in_new={sl.count(raised)}")
raised_lines = raised.splitlines(keepends=True)
diff_idx = [i for i in range(b - a + 1)
            if old_lines[a - 1 + i] != raised_lines[i]]
report("collect_file_metrics region: exactly 1 differing line vs start rev",
       len(diff_idx) == 1, f"differing start-rev line numbers={[a + i for i in diff_idx]}")

# 2. kept non-blank lines survive in order in post-move lib.rs
moved = set()
for a, b in CHUNKS_INDEX + CHUNKS_LOAD:
    moved.update(range(a, b + 1))
kept = [l for i, l in enumerate(old_lines, 1)
        if i not in moved and l.strip()]
new_nonblank = [l for l in lib1.splitlines(keepends=True) if l.strip()]
it = iter(new_nonblank)
subseq = all(any(l == x for x in it) for l in kept)
report("all kept non-blank start-rev lines survive in order in post-move lib.rs",
       subseq, f"kept={len(kept)}")

# 3. line counts
def count(bs):
    return bs.count(b"\n") + (0 if bs.endswith(b"\n") or not bs else 1)
print("COUNTS: lib.rs start-rev", count(old),
      "| lib.rs at 1819b8f", count(lib1),
      "| lib.rs HEAD", count(blob(f"{HEAD}:crates/pipeline/src/lib.rs")),
      "| stage_index.rs at 1819b8f", count(si),
      "| stage_index.rs HEAD", count(blob(f"{HEAD}:crates/pipeline/src/stage_index.rs")),
      "| stage_load.rs at 1819b8f", count(sl),
      "| stage_spans.rs HEAD", count(blob(f"{HEAD}:crates/cli/src/commands/ui/stage_spans.rs")),
      "| ui.rs HEAD", count(blob(f"{HEAD}:crates/cli/src/commands/ui.rs")),
      "| contract_tests start-rev", count(blob(f"{START}:crates/cli/src/commands/ui/progress_contract_tests.rs")),
      "| contract_tests HEAD", count(blob(f"{HEAD}:crates/cli/src/commands/ui/progress_contract_tests.rs")))

# 4. stage_load untouched by commits 2/3
report("stage_load.rs byte-identical 1819b8f -> HEAD", sl == sl_head)

# 5. progress_contract_tests append-only
pct_old = blob(f"{START}:crates/cli/src/commands/ui/progress_contract_tests.rs")
pct_head = blob(f"{HEAD}:crates/cli/src/commands/ui/progress_contract_tests.rs")
po = pct_old.splitlines(keepends=True)
ph = pct_head.splitlines(keepends=True)
report("progress_contract_tests existing prefix byte-identical (append-only)",
       ph[:len(po)] == po,
       f"old_lines={len(po)} head_lines={len(ph)}")

# 6. progress.rs frozen
report("crates/cli/src/commands/progress.rs byte-identical 5c81881 -> HEAD",
       blob(f"{START}:crates/cli/src/commands/progress.rs")
       == blob(f"{HEAD}:crates/cli/src/commands/progress.rs"))

print("VERDICT:", "ALL PROOFS PASS" if not fails else f"FAILURES: {fails}")
