#!/usr/bin/env python3
"""Stage-5 payload-tail bench regression gate (incident E12).

Runs the ignored `payload_tail_bench` bench (`crates/cli/tests/
payload_tail_bench.rs`) N times, parses its `label | ms` phase table,
takes the per-phase median across runs, and compares it against
`tools/bench_baseline.json`, keyed by machine class. This is a
warn-class LOCAL gate (plan D4), not a CI check.

Exit codes:
  0 clean, or no baseline entry exists for this machine class.
  1 some phase median regressed past REGRESSION_RATIO x baseline.
  2 parse format drift, failed/timed-out bench run, bad CLI usage,
    or an internal error (never substitute zeros for missing rows).

Usage:
  python tools/bench_compare.py [--runs N] [--timeout SECONDS]
                                [--baseline PATH] [--update-baseline]

The doc-comment baseline table in the bench source predates plan A1 and
lists stale labels ("clone" vs "hand-out", "plan() cold" vs "plan()
facts+closures"); DOCUMENTED_PHASES below mirrors the rows the CURRENT
bench code actually prints.
"""

import json
import math
import platform
import re
import statistics
import subprocess
import sys
from datetime import date
from pathlib import Path

BENCH_CARGO_ARGS = [
    "cargo", "test", "-p", "ariadne-cli", "--test", "payload_tail_bench",
    "--", "--ignored", "--nocapture", "--test-threads=1",
]

# The exact labels printed by the current payload_tail_bench.rs rows in
# boot order: four load-stage rows, then the ten payload-assembly rows.
DOCUMENTED_PHASES = (
    "symbol_pass+prep (combined, <= first event)",
    "file_pass (event span)",
    "save_load_artifacts+cache (combined)",
    "ensure_load total",
    "get_or_build_include_graph clone (cold)",
    "get_or_build_load hand-out (cold)",
    "get_or_build_include_graph clone (warm)",
    "get_or_build_load hand-out (warm)",
    "generate_report+detect_cycles (combined)",
    "build_include_elements",
    "path_to_id+file_collateral maps (combined)",
    "OptimizationPlanner::plan() facts+closures",
    "conservative filter pass",
    "build_file_justification_with_memo",
)

DEFAULT_RUNS = 3
DEFAULT_TIMEOUT_SECONDS = 900
REGRESSION_RATIO = 1.25
BASELINE_PATH = Path(__file__).resolve().with_name("bench_baseline.json")
NO_BASELINE_HINT = "no baseline for machine class"

PROCESSOR_NON_ALNUM = re.compile(r"[^A-Za-z0-9]+")


class BenchRunError(Exception):
    """The cargo bench invocation failed or its baseline file is broken."""


class ParseDriftError(Exception):
    """A bench row exists but its value is not a finite number."""


def sanitize_processor(raw_processor: str) -> str:
    cleaned = PROCESSOR_NON_ALNUM.sub("_", raw_processor).strip("_")
    return cleaned if cleaned else "cpu"


def machine_class() -> str:
    processor = platform.processor() or "cpu"
    return "-".join([platform.system(), platform.machine(),
                     sanitize_processor(processor)])


def parse_bench_table(text: str) -> dict:
    phases: dict = {}
    for line in text.splitlines():
        if line.count("|") != 1:
            continue
        label, value = (part.strip() for part in line.split("|"))
        if not label or not value:
            continue
        if label.lower() == "phase" and value == "ms":
            continue
        try:
            milliseconds = float(value)
        except ValueError:
            raise ParseDriftError(
                f"parse format drift: unparseable bench row: {line.strip()}"
            ) from None
        if not math.isfinite(milliseconds):
            raise ParseDriftError(
                f"parse format drift: non-finite bench value: {line.strip()}"
            )
        phases[label] = milliseconds
    return phases


def find_missing_documented_phases(runs_output) -> list:
    if not runs_output:
        return list(DOCUMENTED_PHASES)
    return [phase for phase in DOCUMENTED_PHASES
            if any(phase not in run for run in runs_output)]


def medians_per_phase(runs_output) -> dict:
    observed = sorted({phase for run in runs_output for phase in run})
    extra = [phase for phase in observed if phase not in DOCUMENTED_PHASES]
    medians: dict = {}
    for phase in list(DOCUMENTED_PHASES) + extra:
        samples = [run[phase] for run in runs_output if phase in run]
        if samples:
            medians[phase] = statistics.median(samples)
    return medians


def run_bench(runs: int, timeout_seconds: float, cwd: Path) -> list:
    collected = []
    for run_index in range(runs):
        try:
            finished = subprocess.run(
                BENCH_CARGO_ARGS, cwd=str(cwd), capture_output=True,
                text=True, errors="replace", timeout=timeout_seconds,
            )
        except subprocess.TimeoutExpired:
            raise BenchRunError(
                f"bench run {run_index + 1} of {runs} exceeded "
                f"{timeout_seconds} s"
            ) from None
        except OSError as launch_failure:
            raise BenchRunError(f"could not launch cargo: {launch_failure}") from None
        if finished.returncode != 0:
            tail = (finished.stdout + finished.stderr)[-1200:]
            raise BenchRunError(
                f"cargo bench run {run_index + 1} of {runs} failed "
                f"(exit {finished.returncode}):\n{tail}"
            )
        collected.append(parse_bench_table(finished.stdout))
    return collected


def load_baseline(path: Path) -> dict:
    if not path.is_file():
        return {}
    try:
        loaded = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as failure:
        raise BenchRunError(f"cannot read baseline {path}: {failure}") from None
    if not isinstance(loaded, dict):
        raise BenchRunError(f"baseline {path} must contain a JSON object")
    return loaded


def build_baseline_entry(medians: dict, runs: int, recorded: str) -> dict:
    return {
        "phases": {phase: float(ms) for phase, ms in medians.items()},
        "recorded": recorded,
        "runs": runs,
    }


def save_baseline(path: Path, baseline: dict) -> None:
    text = json.dumps(baseline, indent=2, sort_keys=True) + "\n"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def render_class_snapshot(class_key: str, medians: dict, runs: int,
                          recorded: str) -> str:
    snapshot = {class_key: build_baseline_entry(medians, runs, recorded)}
    return json.dumps(snapshot, indent=2, sort_keys=True)


def compare_medians(medians: dict, baseline_phases: dict):
    warnings = []
    regressed = False
    for phase, median_ms in medians.items():
        if phase not in baseline_phases:
            warnings.append(
                f"WARN untracked phase: {phase} (no baseline entry)")
            continue
        baseline_ms = float(baseline_phases[phase])
        if median_ms <= REGRESSION_RATIO * baseline_ms:
            continue
        regressed = True
        if baseline_ms > 0:
            increase = f"+{(median_ms / baseline_ms - 1.0) * 100.0:.0f}%"
        else:
            increase = "+inf%"
        warnings.append(
            f"WARN {phase}: {median_ms:.1f} ms vs baseline "
            f"{baseline_ms:.1f} ms ({increase})")
    return warnings, regressed


def print_median_table(medians: dict) -> None:
    print(f"{'phase':<48} | {'median ms':>9}")
    print("-" * 62)
    for phase, median_ms in medians.items():
        print(f"{phase:<48} | {median_ms:>9.1f}")


def _positive_int(raw: str, option: str) -> int:
    try:
        value = int(raw)
    except ValueError:
        raise ValueError(f"{option} requires an integer, got {raw!r}") from None
    if value < 1:
        raise ValueError(f"{option} must be >= 1, got {value}")
    return value


def _positive_float(raw: str, option: str) -> float:
    try:
        value = float(raw)
    except ValueError:
        raise ValueError(f"{option} requires a number, got {raw!r}") from None
    if value <= 0:
        raise ValueError(f"{option} must be > 0, got {value}")
    return value


def _parse_arguments(arguments):
    runs = DEFAULT_RUNS
    timeout_seconds = float(DEFAULT_TIMEOUT_SECONDS)
    baseline_path = BASELINE_PATH
    update_baseline = False
    index = 0
    while index < len(arguments):
        argument = arguments[index]
        if argument == "--runs":
            index += 1
            if index >= len(arguments):
                raise ValueError("--runs requires a value")
            runs = _positive_int(arguments[index], "--runs")
        elif argument == "--timeout":
            index += 1
            if index >= len(arguments):
                raise ValueError("--timeout requires a value")
            timeout_seconds = _positive_float(arguments[index], "--timeout")
        elif argument == "--baseline":
            index += 1
            if index >= len(arguments):
                raise ValueError("--baseline requires a path")
            baseline_path = Path(arguments[index])
        elif argument == "--update-baseline":
            update_baseline = True
        else:
            raise ValueError(f"unknown argument {argument!r}")
        index += 1
    return runs, timeout_seconds, baseline_path, update_baseline


def _report_missing_phases(missing: list) -> None:
    print("ERROR parse format drift: documented phase row(s) missing from "
          "bench output (never substituting zeros):")
    for phase in missing:
        print(f"  missing documented phase: {phase}")


def _handle_update(medians: dict, runs: int, baseline_path: Path,
                   class_key: str) -> int:
    try:
        baseline = load_baseline(baseline_path)
    except BenchRunError as failure:
        print(f"ERROR {failure}")
        return 2
    baseline[class_key] = build_baseline_entry(
        medians, runs, date.today().isoformat())
    try:
        save_baseline(baseline_path, baseline)
    except OSError as failure:
        print(f"ERROR could not write baseline {baseline_path}: {failure}")
        return 2
    print(f"baseline entry updated for machine class '{class_key}' "
          f"in {baseline_path}")
    return 0


def _handle_compare(medians: dict, runs: int, baseline_path: Path,
                    class_key: str) -> int:
    try:
        baseline = load_baseline(baseline_path)
    except BenchRunError as failure:
        print(f"ERROR {failure}")
        return 2
    entry = baseline.get(class_key)
    if entry is None:
        print(f"WARNING {NO_BASELINE_HINT} '{class_key}'; regression compare "
              "skipped. Ready-to-paste baseline JSON:")
        print(render_class_snapshot(class_key, medians, runs,
                                    date.today().isoformat()))
        return 0
    baseline_phases = entry.get("phases") if isinstance(entry, dict) else None
    if not isinstance(baseline_phases, dict):
        print(f"ERROR baseline entry for '{class_key}' has no \"phases\" "
              f"object in {baseline_path}")
        return 2
    warnings, regressed = compare_medians(medians, baseline_phases)
    for warning in warnings:
        print(warning)
    if regressed:
        return 1
    print(f"bench compare: {runs} run(s), no regressions past "
          f"{REGRESSION_RATIO:.2f}x baseline")
    return 0


def main(argv=None, run_bench_impl=None) -> int:
    bench_runner = run_bench_impl or run_bench
    arguments = list(sys.argv[1:] if argv is None else argv)
    try:
        runs, timeout_seconds, baseline_path, update_baseline = (
            _parse_arguments(arguments))
    except ValueError as usage_error:
        print(f"ERROR {usage_error}")
        return 2
    repo_root = Path(__file__).resolve().parents[1]
    try:
        runs_output = bench_runner(runs, timeout_seconds, repo_root)
    except (BenchRunError, ParseDriftError) as run_failure:
        print(f"ERROR {run_failure}")
        return 2
    missing = find_missing_documented_phases(runs_output)
    if missing:
        _report_missing_phases(missing)
        return 2
    medians = medians_per_phase(runs_output)
    print_median_table(medians)
    class_key = machine_class()
    if update_baseline:
        return _handle_update(medians, runs, baseline_path, class_key)
    return _handle_compare(medians, runs, baseline_path, class_key)


if __name__ == "__main__":
    sys.exit(main())
