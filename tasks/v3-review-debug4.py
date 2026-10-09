"""Debug pass 4: opcode-level diff of lib.rs start-rev -> 1819b8f.
Equal spans as counts; deletions OUTSIDE the five declared moved chunks shown
in full; all additions shown in full. Ephemeral; delete after run."""
import subprocess
import difflib

def blob(rev_path):
    return subprocess.check_output(["git", "cat-file", "blob", rev_path])

old = blob("5c81881:crates/pipeline/src/lib.rs").splitlines(keepends=True)
new = blob("1819b8f:crates/pipeline/src/lib.rs").splitlines(keepends=True)

CHUNKS_INDEX = [(382, 537), (779, 798), (822, 891)]
CHUNKS_LOAD = [(617, 741), (893, 952)]
ALL = CHUNKS_INDEX + CHUNKS_LOAD
moved = set()
for a, b in ALL:
    moved.update(range(a, b + 1))

sm = difflib.SequenceMatcher(None, old, new, autojunk=False)
stray_del = []
adds = []
eq_total = 0
del_in_chunks = 0
for tag, i1, i2, j1, j2 in sm.get_opcodes():
    if tag == "equal":
        eq_total += i2 - i1
        continue
    if tag in ("replace", "delete"):
        for k in range(i1, i2):
            if (k + 1) in moved:
                del_in_chunks += 1
            else:
                stray_del.append((k + 1, old[k]))
    if tag in ("replace", "insert"):
        for k in range(j1, j2):
            adds.append((k + 1, new[k]))

print(f"equal lines total: {eq_total}")
print(f"deletions inside declared moved chunks: {del_in_chunks}")
print(f"STRAY deletions outside declared chunks: {len(stray_del)}")
for i, l in stray_del:
    print(f"  STRAY-DEL old:{i}: {l[:100]!r}")
print(f"ADDITIONS in commit-1 lib.rs: {len(adds)}")
for i, l in adds:
    print(f"  ADD new:{i}: {l[:110]!r}")
