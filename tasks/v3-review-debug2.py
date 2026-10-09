"""Debug pass 2: locate the collect_file_metrics declaration, first failing
kept line in the subsequence check, and exact opcodes between the contract
test blobs. Ephemeral review artifact; delete after running."""
import subprocess
import difflib

def blob(rev_path):
    return subprocess.check_output(["git", "cat-file", "blob", rev_path])

START = "5c81881"
C1 = "1819b8f"
HEAD = "1e81d7e"

old = blob(f"{START}:crates/pipeline/src/lib.rs")
old_lines = old.splitlines(keepends=True)
sl = blob(f"{C1}:crates/pipeline/src/stage_load.rs")
sl_lines = sl.splitlines(keepends=True)
lib1 = blob(f"{C1}:crates/pipeline/src/lib.rs")

print("== collect_file_metrics declaration lines in start-rev lib.rs ==")
for i, l in enumerate(old_lines, 1):
    if b"collect_file_metrics" in l:
        print(f"  old lib.rs:{i}: {l!r}")
print("== collect_file_metrics declaration lines in stage_load.rs ==")
for i, l in enumerate(sl_lines, 1):
    if b"collect_file_metrics" in l:
        print(f"  stage_load.rs:{i}: {l!r}")
print("== lines around start-rev 893-895 and 890-892, 950-959 ==")
for i in range(889, 896):
    print(f"  old:{i}: {old_lines[i-1]!r}")
for i in range(950, min(960, len(old_lines)) + 1):
    print(f"  old:{i}: {old_lines[i-1]!r}")
print("== stage_load.rs first 12 lines and last 6 ==")
for i in range(1, 13):
    print(f"  sl:{i}: {sl_lines[i-1]!r}")
for i in range(len(sl_lines) - 5, len(sl_lines) + 1):
    print(f"  sl:{i}: {sl_lines[i-1]!r}")

CHUNKS_INDEX = [(382, 537), (779, 798), (822, 891)]
CHUNKS_LOAD = [(617, 741), (893, 952)]
moved = set()
for a, b in CHUNKS_INDEX + CHUNKS_LOAD:
    moved.update(range(a, b + 1))
kept = [(i, l) for i, l in enumerate(old_lines, 1)
        if i not in moved and l.strip()]
new_lines = lib1.splitlines(keepends=True)
new_set = {}
for j, l in enumerate(new_lines):
    new_set.setdefault(l, []).append(j + 1)
print("== first 5 kept start-rev lines absent from post-move lib.rs ==")
misses = [(i, l) for i, l in kept if l not in new_set]
for i, l in misses[:5]:
    print(f"  MISSING old:{i}: {l!r}")
print(f"  total missing kept lines: {len(misses)} of {len(kept)}")

print("== order check among surviving kept lines ==")
survivors = [(i, l) for i, l in kept if l in new_set]
pos = []
ok = True
last_used = -1
for i, l in survivors:
    cands = new_set[l]
    nxt = next((p for p in cands if p > last_used), None)
    if nxt is None:
        print(f"  ORDER VIOLATION at old:{i}: {l!r}")
        ok = False
        break
    last_used = nxt
print("  order OK" if ok else "  order broken (see above)")

print("== contract test file opcodes (old 305 -> HEAD) ==")
po = blob(f"{START}:crates/cli/src/commands/ui/progress_contract_tests.rs").splitlines()
ph = blob(f"{HEAD}:crates/cli/src/commands/ui/progress_contract_tests.rs").splitlines()
sm = difflib.SequenceMatcher(None, po, ph, autojunk=False)
for tag, i1, i2, j1, j2 in sm.get_opcodes():
    if tag != "equal":
        print(f"  {tag} old[{i1}:{i2}] new[{j1}:{j2}]")
        for x in po[i1:i2][:3]:
            print(f"    - {x!r}")
        for x in ph[j1:j2][:4]:
            print(f"    + {x!r}")
print("== old file last 3 lines ==")
for x in po[-3:]:
    print(f"  ~ {x!r}")
