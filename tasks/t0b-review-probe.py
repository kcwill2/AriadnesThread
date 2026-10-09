"""M28 T0b reviewer scratch probe -- operator purge after verdict.

Read-only witnesses for the t0b-gate-mode review: disk precondition
(ledger row 70), mode constants, baseline key/JSON state, and two
independent driver scenarios fed through bench_compare.main with an
injected fake runner (cargo never runs, bench_baseline.json never
touched): (A) synthetic ACCEPTED in-band memory output against a
synthetic temp-dir baseline -> expect exit 0 with the bars named in the
pass line; (B) same output with the full peak median pushed over the
RATIFIED 5200 MiB bar -> expect exit 1 with the RATIFIED warn line.
"""
import io
import json
import shutil
import sys
import tempfile
from contextlib import redirect_stdout
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import bench_compare

total, used, free = shutil.disk_usage("C:\\")
print("DISK C: free %.1f GiB of %.1f GiB total" % (free / 2**30, total / 2**30))

print("default runs (timing):", bench_compare._parse_arguments([])[0])
print("default runs (memory):",
      bench_compare._parse_arguments(["--bench", "stage5-memory"])[0])
print("memory key:", bench_compare.stage5_memory_baseline_key())
print("tuple length:", len(bench_compare.DOCUMENTED_PHASES_STAGE5_MEMORY))
print("peak labels:", bench_compare.STAGE5_MEMORY_PEAK_LABELS)
print("ratio:", bench_compare.STAGE5_MEMORY_REGRESSION_RATIO,
      "| bar:", bench_compare.STAGE5_MEMORY_PEAK_BAR_MIB,
      "| gate G:", bench_compare.STAGE5_MEMORY_GROWTH_GATE_RATIO,
      "| divergence:", bench_compare.STAGE5_MEMORY_DIVERGENCE_FRACTION)

baseline = json.load(open("tools/bench_baseline.json", encoding="utf-8"))
print("baseline keys:", sorted(baseline))
print("stage5-memory keys present:",
      [key for key in baseline if key.endswith("::stage5-memory")])

HALF = "stage5 peak RSS (debug, N=640)"
FULL = "stage5 peak RSS (debug, N=1280)"
RATIO = "stage5 RSS growth ratio (debug, N=640->1280)"


def render_row(label, value):
    return "%-64s | %12.1f" % (label, value)


def synthetic_output(overrides=None):
    values = dict(overrides or {})
    lines = ["running 8 tests", "-" * 79]
    for label in bench_compare.DOCUMENTED_PHASES_STAGE5_MEMORY:
        if label == HALF:
            value = values.get(HALF, 2400.0)
        elif label == FULL:
            value = values.get(FULL, 4600.0)
        elif label == RATIO:
            value = values.get(RATIO, 1.9)
        else:
            value = values.get(label, 42.0)
        lines.append(render_row(label, value))
    for label in (HALF, FULL):
        lines.append(render_row(label, values.get(label, 2400.0 if label == HALF else 4600.0)))
    lines.append("test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured")
    return "\n".join(lines) + "\n"


def make_runner(outputs):
    def runner(requested_runs, timeout_seconds, repo_root):
        return [bench_compare.parse_bench_table_multiset(text)
                for text in outputs]
    return runner


def drive(outputs, baseline_doc):
    with tempfile.TemporaryDirectory() as scratch:
        path = Path(scratch) / "bench_baseline.json"
        path.write_text(json.dumps(baseline_doc), encoding="utf-8")
        stream = io.StringIO()
        with redirect_stdout(stream):
            code = bench_compare.main(
                ["--bench", "stage5-memory", "--baseline", str(path)],
                run_bench_impl=make_runner(outputs))
        return code, stream.getvalue()


key = bench_compare.stage5_memory_baseline_key()
synthetic_phases = {}
for label in bench_compare.DOCUMENTED_PHASES_STAGE5_MEMORY:
    if label == HALF:
        synthetic_phases[label] = 2400.0
    elif label == FULL:
        synthetic_phases[label] = 4600.0
    elif label == RATIO:
        synthetic_phases[label] = 1.9
    else:
        synthetic_phases[label] = 42.0
synthetic_entry = {"phases": synthetic_phases,
                   "recorded": "2026-10-06", "runs": 5}

code, out = drive([synthetic_output() for _ in range(5)], {key: synthetic_entry})
print("SCENARIO A exit:", code)
print("SCENARIO A verdict line:",
      [line for line in out.splitlines() if "regressions" in line or "WARN" in line or "accounting" in line])

code, out = drive([synthetic_output({FULL: 5300.0}) for _ in range(5)],
                  {key: synthetic_entry})
print("SCENARIO B exit:", code)
for line in out.splitlines():
    if "RATIFIED" in line or "WARN" in line:
        print("SCENARIO B:", line)

outputs_c = [synthetic_output() for _ in range(5)]
# Invocation 4 carries a +25% half-scale peak inflation on BOTH its
# samples (curve child + ratio-child re-emit): must be rejected
# wholesale, loudly, while the 5 clean invocations still gate.
outputs_c.insert(3, synthetic_output({HALF: 3000.0}))
code, out = drive(outputs_c, {key: synthetic_entry})
print("SCENARIO C (one inflated invocation) exit:", code)
for line in out.splitlines():
    if "REJECT" in line or "accounting" in line:
        print("SCENARIO C:", line)
