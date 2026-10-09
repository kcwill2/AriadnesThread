"""M28 T4b reviewer scratch probe - live-shape tripwire-vs-bar witnesses.

Drives bench_compare.main() through the REAL compare path (parse -> sample
law -> medians -> bars/tripwire -> exit code) with an injected fake bench
runner (the sanctioned test seam, run_bench_impl) and a temp-dir baseline.
tools/bench_baseline.json is NEVER referenced: every call passes an
explicit --baseline path inside a TemporaryDirectory.

Cases:
  A ratio 4.00 (in-band peak) -> exit 0 + TRIPWIRE line naming the user
    decision; baseline ratio kept 4.0/1.4 so the 1.50x row check passes
    (isolates the demoted ABSOLUTE gate from baseline-row semantics).
  B peak 5300 (canonical ratio) -> exit 1 + the RATIFIED peak-bar WARN.
  C ratio exactly 2.0 -> exit 0 + NO TRIPWIRE line (strict > boundary).
  D ratio 4.00, ratio row ABSENT from baseline -> exit 0 + TRIPWIRE
    (tripwire independent of baseline presence).
"""
import sys
import tempfile
from contextlib import redirect_stdout
from io import StringIO
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "tools"))

import bench_compare  # noqa: E402
import bench_compare_tests as T  # noqa: E402


def run_case(name, baseline, texts, arguments):
    with tempfile.TemporaryDirectory() as scratch:
        path = T.write_baseline_file(scratch, baseline)
        exit_code, output, _ = T.invoke_memory_main(
            ["--baseline", str(path), *arguments], texts)
    tripwires = [line for line in output.splitlines() if "TRIPWIRE" in line]
    warns = [line for line in output.splitlines()
             if line.startswith("WARN") and "peak RSS" in line]
    print("== %s: exit=%d" % (name, exit_code))
    for line in tripwires + warns:
        print("   %s" % line)
    return exit_code, output


def main():
    ratio_label = T.GROWTH_RATIO_LABEL
    full_peak = T.FULL_PEAK_LABEL

    exit_a, out_a = run_case(
        "A ratio 4.00 in-band",
        T.memory_baseline_with_entry({ratio_label: 4.0 / 1.4}),
        T.memory_texts(5, uniform={"value_overrides":
                                   {ratio_label: 4.0}}), [])
    assert exit_a == 0, "A must exit 0 (tripwire never fails the gate)"
    assert ("TRIPWIRE " + ratio_label) in out_a, "A must print TRIPWIRE"
    assert "user decision 2026-10-06" in out_a, "A cites the decision"

    exit_b, out_b = run_case(
        "B peak 5300 exceeds bar",
        T.memory_baseline_with_entry({full_peak: 3600.0}),
        T.memory_texts(5, uniform={"value_overrides":
                                   {full_peak: 5300.0}}), [])
    assert exit_b == 1, "B must exit 1 (RATIFIED peak bar still gates)"
    assert "TRIPWIRE" not in out_b, "B: canonical ratio, no tripwire"

    exit_c, _ = run_case(
        "C ratio exactly 2.0 boundary",
        T.memory_baseline_with_entry({ratio_label: 2.0 / 1.4}),
        T.memory_texts(5, uniform={"value_overrides":
                                   {ratio_label: 2.0}}), [])
    assert exit_c == 0, "C exits 0"
    exit_c2, out_c2 = run_case(
        "C2 ratio 2.0, no TRIPWIRE assertion",
        T.memory_baseline_with_entry({ratio_label: 2.0 / 1.4}),
        T.memory_texts(5, uniform={"value_overrides":
                                   {ratio_label: 2.0}}), [])
    assert "TRIPWIRE" not in out_c2, "C2: strict >, no line at exactly 2.0"

    baseline_d = T.memory_baseline_with_entry()
    for entry in baseline_d.values():
        entry["phases"].pop(ratio_label)
    exit_d, out_d = run_case(
        "D ratio 4.00 absent from baseline",
        baseline_d,
        T.memory_texts(5, uniform={"value_overrides":
                                   {ratio_label: 4.0}}), [])
    assert exit_d == 0, "D exits 0 per baseline rules"
    assert ("TRIPWIRE " + ratio_label) in out_d, "D prints TRIPWIRE anyway"

    print("ALL T4b LIVE-SHAPE WITNESSES PASS")


if __name__ == "__main__":
    main()
