"""Final targeted proofs: (1) ProgressEvent/LoadMilestone/ProgressCb block
lines byte-identical and in-order in HEAD (variants APPENDED, not inserted);
(2) stage_spans.rs 1819b8f->HEAD opcode diff (the single deletion).
Ephemeral; delete after run."""
import subprocess
import difflib

def blob(rev_path):
    return subprocess.check_output(["git", "cat-file", "blob", rev_path])

old = blob("5c81881:crates/pipeline/src/lib.rs").splitlines(keepends=True)
head = blob("1e81d7e:crates/pipeline/src/lib.rs").splitlines(keepends=True)

def block(lines):
    a = next(i for i, l in enumerate(lines)
             if l.startswith(b"#[derive")
             and any(b"ProgressEvent" in x for x in lines[i:i + 2]))
    b = next(i for i, l in enumerate(lines) if l.startswith(b"type ProgressCb"))
    return lines[a:b + 1]

ob, hb = block(old), block(head)
print(f"start-rev event block: {len(ob)} lines; HEAD block: {len(hb)} lines")
sm = difflib.SequenceMatcher(None, ob, hb, autojunk=False)
bad = []
for tag, i1, i2, j1, j2 in sm.get_opcodes():
    if tag == "insert":
        for x in hb[j1:j2]:
            print(f"  + inserted-after-old-line-{i1}: {x[:80]!r}")
    elif tag != "equal":
        bad.append((tag, i1, i2, j1, j2))
        print(f"  NON-INSERT op {tag} old[{i1}:{i2}] new[{j1}:{j2}]")
        for x in ob[i1:i2]:
            print(f"    - {x[:80]!r}")
        for x in hb[j1:j2]:
            print(f"    + {x[:80]!r}")
print("ENUM BLOCK:", "APPEND-ONLY (all start-rev lines byte-identical, in order)"
      if not bad else f"VIOLATIONS: {bad}")

ss1 = blob("1819b8f:crates/cli/src/commands/ui/stage_spans.rs").splitlines()
ssh = blob("1e81d7e:crates/cli/src/commands/ui/stage_spans.rs").splitlines()
print(f"stage_spans 1819b8f {len(ss1)} lines -> HEAD {len(ssh)} lines; opcodes:")
for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, ss1, ssh, autojunk=False).get_opcodes():
    if tag != "equal":
        print(f"  {tag} old[{i1}:{i2}] new[{j1}:{j2}]")
        for x in ss1[i1:i2]:
            print(f"    - {x[:90]!r}")
        for x in ssh[j1:j2][:6]:
            print(f"    + {x[:90]!r}")
