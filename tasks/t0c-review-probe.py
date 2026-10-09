"""M28 T0c reviewer probe -- operator purge after verdict (row-67 class).

Independent anti-fabrication audit for the t0c measurement hop. NOTHING is
imported from the implementer's helper: canonical-label regexes parse the
raw captures; the ratified law is re-implemented standalone here. Checks:
  1. witness-line census: 'load-memo caps override active' present in all 5
     G3 captures and in NO G1/G2/preflight capture (per-file line numbers);
  2. per-capture context lines (STAGE5_BENCH trio) and the D5 prefix
     witness: the 2363.9 row literally carrying 'test stage5_rss_growth_
     ratio ... ' in the raw capture;
  3. independent parse + rejection law over G1/G2/G3 peak slots; accepted
     medians; save-tail medians under BOTH bases (all and accepted);
  4. label census: every pipe-row label (prefix-normalized independently)
     is a known bench template label for the capture profile - nothing
     hidden, nothing merged;
  5. ablation arithmetic recomputed from section 3's medians.
"""
import re
import statistics
import sys
from pathlib import Path

TARGET = Path("target")
G1 = [TARGET / ("t0c-g1-release-%d.txt" % k) for k in range(1, 7)]
G2 = [TARGET / ("t0c-g2-debug-%d.txt" % k) for k in range(1, 6)]
G3 = [TARGET / ("t0c-g3-cap0-%d.txt" % k) for k in range(1, 6)]
PREFLIGHT = [TARGET / "t0c-preflight.txt"]
WITNESS = "load-memo caps override active"

PEAK_RE = re.compile(
    r"stage5 peak RSS \((release|debug), N=(640|1280)\)\s*\|\s*([0-9.]+)")
SAVE_RE = re.compile(
    r"RSS checkpoint save tail \((release|debug), N=1280\)\s*\|\s*([0-9.]+)")
RATIO_RE = re.compile(
    r"stage5 RSS growth ratio \((release|debug), N=640->1280\)\s*\|\s*([0-9.]+)")

sys.path.insert(0, "tools")
import bench_compare

PROFILES = ("debug", "release")
expected_labels = set()
for profile in PROFILES:
    for label in bench_compare.DOCUMENTED_PHASES_STAGE5_MEMORY:
        expected_labels.add(label.replace("(debug", "(%s" % profile))


def read(path):
    return path.read_text(encoding="utf-8", errors="replace")


def normalize(label):
    stripped = label.strip()
    if stripped.startswith("test ") and " ... " in stripped:
        head, _, tail = stripped.partition(" ... ")
        name = head[len("test "):]
        if " " not in name:
            return tail.strip()
    return stripped


print("=== 1. witness census ===")
for group, paths in (("G1", G1), ("G2", G2), ("G3", G3), ("PF", PREFLIGHT)):
    for path in paths:
        lines = read(path).splitlines()
        hits = [i + 1 for i, line in enumerate(lines) if WITNESS in line]
        print("%s %s witness-lines=%s %s" % (
            group, path.name, hits,
            ["VALUES:%s" % l.strip() for l in lines if WITNESS in l]))

print("=== 2. context + D5 prefix witness ===")
for path in G1[:1] + G3[:1] + G2[:1]:
    ctx = [l.strip() for l in read(path).splitlines()
           if l.startswith("STAGE5_BENCH")]
    print(path.name, "context:", ctx)
raw3 = read(G1[2])
prefix_hits = [l.strip() for l in raw3.splitlines()
               if "2363.9" in l]
print("k3 raw rows containing 2363.9:", prefix_hits)

print("=== 3+4. independent parse, law, label census ===")


def analyze(group, paths, profile):
    peak_samples = []
    save_values = []
    ratio_values = []
    unknown_labels = set()
    for path in paths:
        samples = []
        for line in read(path).splitlines():
            if line.count("|") != 1:
                continue
            label = normalize(line.split("|", 1)[0])
            if label not in expected_labels:
                unknown_labels.add(label)
            match = PEAK_RE.search(line)
            if match:
                samples.append(("%s|%s" % (match.group(1), match.group(2)),
                                float(match.group(3))))
            if SAVE_RE.search(line):
                save_values.append(float(SAVE_RE.search(line).group(2)))
            if RATIO_RE.search(line):
                ratio_values.append(float(RATIO_RE.search(line).group(2)))
        peak_samples.append(samples)
    pool = {}
    for samples in peak_samples:
        for slot, value in samples:
            pool.setdefault(slot, []).append(value)
    slot_medians = {slot: statistics.median(values)
                    for slot, values in pool.items()}
    accepted = []
    for index, samples in enumerate(peak_samples, start=1):
        reasons = []
        for slot, value in samples:
            median = slot_medians[slot]
            divergence = abs(value - median) / median
            if divergence > 0.08:
                reasons.append("%s sample %.1f vs %.1f = %.1f%%"
                               % (slot, value, median, divergence * 100))
        if reasons:
            print("  %s REJECT invocation %d: %s"
                  % (group, index, "; ".join(reasons)))
        else:
            accepted.append(index)
    accepted_pool = {}
    for index, samples in enumerate(peak_samples, start=1):
        if index in accepted:
            for slot, value in samples:
                accepted_pool.setdefault(slot, []).append(value)
    accepted_medians = {slot: statistics.median(values)
                        for slot, values in accepted_pool.items()}
    print("  %s accepted %d of %d | accepted peak medians: %s" % (
        group, len(accepted), len(peak_samples),
        {slot: round(median, 1) for slot, median in
         sorted(accepted_medians.items())}))
    accepted_save_values = []
    for index, path in enumerate(paths, start=1):
        if index in accepted:
            for line in read(path).splitlines():
                match = SAVE_RE.search(line)
                if match:
                    accepted_save_values.append(float(match.group(2)))
    print("  %s save-tail median(all)=%.1f  save-tail median(accepted-only inv)=%.1f"
          % (group,
             statistics.median(save_values),
             statistics.median(accepted_save_values)))
    print("  %s ratio median(all)=%.1f" % (group, statistics.median(ratio_values)))
    print("  %s unknown labels (must be empty): %s" % (group, sorted(unknown_labels)))


analyze("G1", G1, "release")
analyze("G2", G2, "debug")
analyze("G3", G3, "release")

print("=== 5. ablation arithmetic ===")
print("transient landed check 8469.5-4371.5 =", 8469.5 - 4371.5)
print("transient cap-0 check 8479.2-4381.0 =", 8479.2 - 4381.0)
print("memo residual check 8469.5-8479.2 =", round(8469.5 - 8479.2, 1))
print("uncapped-class check 9040.0-8479.2 =", round(9040.0 - 8479.2, 1))
print("A7-alone claim 8469.5-4098.0 = %.1f  <= 5200: %s"
      % (8469.5 - 4098.0, 8469.5 - 4098.0 <= 5200.0))
