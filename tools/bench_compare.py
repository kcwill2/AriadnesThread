#!/usr/bin/env python3
"""Stage-5 bench regression gates (incident E12 timing; M28 T0b memory).

Modes, selected with --bench (default stage5-timing):

  stage5-timing: runs the ignored `payload_tail_bench` bench
    (`crates/cli/tests/payload_tail_bench.rs`) N times, parses its
    `label | ms` phase table, takes the per-phase median across runs,
    and compares it against `tools/bench_baseline.json`, keyed by
    machine class. This is a warn-class LOCAL gate (plan D4), not a CI
    check. Its parse contract stays strictly one row per label and is
    unchanged by the memory mode.

  stage5-memory: runs the ignored `stage5_memory_bench`
    (crates/cli/tests/stage5_memory_bench.rs, debug profile per today's
    default run mechanics) N times (default 5), parses `label | value`
    rows against DOCUMENTED_PHASES_STAGE5_MEMORY, collects a MULTISET
    per label (peak labels occur twice per scale in suite runs - the
    curve child + the ratio child's re-emitted operand: two honest
    samples of one quantity, T0a verdict finding 1a), rejects every
    invocation whose peak-row operand diverges > 8% from the per-slot
    median (RATIFIED 2026-10-06 DP-4 protocol, binding on both operands
    at both scales, in either direction), takes per-label medians over
    >= 5 ACCEPTED invocations, and compares each row against its own
    baseline key namespace using STAGE5_MEMORY_REGRESSION_RATIO plus the
    RATIFIED absolute peak bar and the growth tripwire (demoted from a
    gate by user decision 2026-10-06). UNITS: RSS/size rows are MiB,
    wall rows seconds, memo rows counts. LOCAL gate only (DP-3): no
    wall-time or RSS number ever runs in CI.

Exit codes:
  0 clean, or no baseline entry exists for this machine class.
  1 some row median regressed past the mode's regression ratio x
    baseline, or a stage5-memory median exceeds the RATIFIED PEAK bar
    (5200 MiB at N=1280). The stage5 growth ratio is a TRIPWIRE (user
    decision 2026-10-06): it is computed and printed and NEVER fails a
    gate.
  2 parse format drift, failed bench run, bad CLI usage, an
    internal error (never substitute zeros for missing rows), or
    stage5-memory accepting fewer than 5 invocations (not a verdict).

Usage:
  python tools/bench_compare.py [--bench {stage5-timing,stage5-memory}]
                                [--runs N] [--timeout SECONDS]
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

STAGE5_MEMORY_CARGO_ARGS = [
    "cargo", "test", "-p", "ariadne-cli", "--test", "stage5_memory_bench",
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

# --- stage5-memory gate mode (M28 T0b) -------------------------------------
BENCH_MODE_TIMING = "stage5-timing"
BENCH_MODE_MEMORY = "stage5-memory"
BENCH_MODES = (BENCH_MODE_TIMING, BENCH_MODE_MEMORY)

# Gate profile and scales: RATIFIED 2026-10-06 at the DP-4 evidence review
# (tasks/stage5-collateral-memory-ceiling-t0a-evidence.md sec.
# "RATIFICATION + ROUND-2 ERRATUM"): N0 = 1280 fixture files, scales
# {640, 1280}. The machine gate observes the debug profile only -- the
# release column of the T0a evidence table is evidence-review data, not a
# gate run the machine re-executes (T0b handoff, "Objective and
# invariants"). Labels carry the profile, so the tuple below is
# debug-only by construction; its exactness against the bench source is
# pinned by bench_compare_tests.Stage5MemoryTupleExactnessTests.
STAGE5_MEMORY_PROFILE = "debug"
STAGE5_MEMORY_SCALES = (640, 1280)
# Label stems exactly as stage5_memory_bench.rs prints them with its
# per-scale suffix (20 scale-parameterised templates).
STAGE5_MEMORY_SCALE_ROW_TEMPLATES = (
    "RSS checkpoint prep",
    "RSS checkpoint symbol-pass end",
    "RSS checkpoint file-pass tail",
    "RSS checkpoint save tail",
    "RSS checkpoint post-return",
    "symbol-pass RSS delta",
    "file-pass RSS delta",
    "save-tail RSS delta",
    "post-return RSS delta",
    "stage5 peak RSS",
    "boot stages 1-4 wall seconds",
    "ensure_load wall seconds",
    "memo semantic-closure entries",
    "memo semantic-closure sampled seeds",
    "memo semantic-closure sampled entry size",
    "memo semantic-closure est aggregate refs",
    "memo physical-expansion entries",
    "memo physical-expansion sampled files",
    "memo physical-expansion sampled entry size",
    "memo physical-expansion est aggregate refs",
)
# Label stems the bench prints with a profile-only suffix (warm rows).
STAGE5_MEMORY_WARM_ROW_TEMPLATES = (
    "warm deserialize peak",
    "warm deserialize delta",
    "warm deserialize wall seconds",
    "warm report.bin size",
    "warm report.json size",
)


def _stage5_memory_labels() -> tuple:
    labels = []
    for scale in STAGE5_MEMORY_SCALES:
        suffix = f"({STAGE5_MEMORY_PROFILE}, N={scale})"
        labels.extend(f"{template} {suffix}"
                      for template in STAGE5_MEMORY_SCALE_ROW_TEMPLATES)
    half_scale, full_scale = STAGE5_MEMORY_SCALES
    labels.append(
        f"stage5 RSS growth ratio ({STAGE5_MEMORY_PROFILE}, "
        f"N={half_scale}->{full_scale})")
    warm_suffix = f"({STAGE5_MEMORY_PROFILE})"
    labels.extend(f"{template} {warm_suffix}"
                  for template in STAGE5_MEMORY_WARM_ROW_TEMPLATES)
    return tuple(labels)


# The 46 concrete rows one full-suite invocation prints on the debug gate
# profile at the RATIFIED scales: 20 scale-parameterised templates per
# scale (x {640, 1280}), the growth-ratio row, and the 5 warm rows.
DOCUMENTED_PHASES_STAGE5_MEMORY = _stage5_memory_labels()

# The growth-ratio operands: peak rows at BOTH scales (RATIFIED protocol,
# binding on both operands at both scales).
STAGE5_MEMORY_PEAK_LABELS = tuple(
    label for label in DOCUMENTED_PHASES_STAGE5_MEMORY
    if label.startswith("stage5 peak RSS "))
STAGE5_MEMORY_PEAK_FULL_LABEL = (
    f"stage5 peak RSS ({STAGE5_MEMORY_PROFILE}, "
    f"N={STAGE5_MEMORY_SCALES[-1]})")
STAGE5_MEMORY_RATIO_ROW_LABEL = (
    f"stage5 RSS growth ratio ({STAGE5_MEMORY_PROFILE}, "
    f"N={STAGE5_MEMORY_SCALES[0]}->{STAGE5_MEMORY_SCALES[-1]})")

# RATIFIED (2026-10-06 DP-4 evidence review, evidence doc sec.
# "RATIFICATION + ROUND-2 ERRATUM"): peak-RSS bar `stage5 peak RSS`
# <= 5200 MiB per profile at N=1280; this machine gate observes the
# debug profile label.
STAGE5_MEMORY_PEAK_BAR_MIB = 5200.0
# DEMOTED to a tripwire by user decision 2026-10-06 (M28 T4b) from the
# 2026-10-06 DP-4 RATIFIED growth gate G: `stage5 RSS growth ratio`
# <= 2.0. The value 2.0 is the demoted ratified constant; it is now a
# tripwire threshold only (computed, printed, never gate-failing) because
# the fixture is Theta(N^2) in aggregate retained closure by construction
# (t0c-verdict sec. 7): the 4.0-4.1 reading IS the payload shape.
STAGE5_MEMORY_GROWTH_TRIPWIRE_RATIO = 2.0
# RATIFIED (same source): gate protocol is median-of->=5 isolated
# invocations; ANY invocation whose operands diverge > 8% from the
# per-slot median is REJECTED, binding on both operands at both scales,
# in EITHER direction (round-2 erratum: working-set trimming can inflate
# OR deflate either operand under ambient memory pressure). Fewer than
# 5 accepted invocations is not a verdict: this hop pins the explicit
# insufficient-samples exit-2 path (never a silent pass, never a gate
# on <5).
STAGE5_MEMORY_MIN_ACCEPTED_INVOCATIONS = 5
STAGE5_MEMORY_DIVERGENCE_FRACTION = 0.08
STAGE5_MEMORY_DEFAULT_RUNS = STAGE5_MEMORY_MIN_ACCEPTED_INVOCATIONS
# CHOSEN BY THIS HOP -- NOT a ratified constant. The evidence doc's
# "Remain PROPOSED" section: "The 1.15x cross-machine tolerance stays
# PROPOSED: it is owned by T0b gate design, pending PR-2", and the T0b
# handoff requires a regression ratio wider than the ms mode's 1.25x.
# Rationale: RSS rows carry allocator fragmentation and working-set
# trimming the ms timing gate never sees; even after the RATIFIED > 8%
# rejection + median protocol trims per-invocation outliers, the
# observed envelope spans +12% half-scale bimodality and -14% full-side
# trimming (evidence doc E1 and the round-2 erratum) versus a 0.06% /
# 0.13% in-band spread for ms phases, and no second-machine baseline
# exists pre-PR-2 to shrink that uncertainty. 1.50x keeps the gate
# honest across baseline eras while the measured pre-fix class (4.0x
# growth, ~9040 MiB peaks) misses the bar by miles. Exact in binary
# floating point, so boundary rows compare exactly. The landing
# reviewer adjudicates this choice.
STAGE5_MEMORY_REGRESSION_RATIO = 1.50
# Own baseline key namespace: memory rows are keyed like today's
# machine-class keys with a mode suffix, so they never collide with ms
# rows on the same machine class (shape sec. 3.1).
STAGE5_MEMORY_BASELINE_KEY_SUFFIX = "::stage5-memory"

USAGE_TEXT = (
    "usage: python tools/bench_compare.py "
    f"[--bench {{{','.join(BENCH_MODES)}}}] [--runs N] [--timeout SECONDS] "
    "[--baseline PATH] [--update-baseline]\n"
    "  stage5-timing (default): payload_tail_bench ms phase table; its\n"
    "    default run count is 3; values are ms.\n"
    "  stage5-memory: stage5_memory_bench RSS curve gate; its default run\n"
    "    count is 5 (the RATIFIED minimum of accepted invocations); values\n"
    "    are MiB (wall rows seconds, memo rows counts).\n"
    "exit: 0 clean or no baseline entry for this machine class;\n"
    "      1 regression past the mode ratio, or (stage5-memory) a\n"
    "        RATIFIED peak-bar exceedance; the growth tripwire prints\n"
    "        but never fails the exit code;\n"
    "      2 parse drift, failed bench run, bad usage, or (stage5-memory)\n"
    "        fewer than 5 accepted invocations."
)


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


def _parse_pipe_row(line: str):
    """Return (label, value) for one shared-contract `label | number` row,
    or None for a line the grammar skips. A pipe row with an unparseable
    or non-finite value raises ParseDriftError -- the same drift contract
    both modes share."""
    if line.count("|") != 1:
        return None
    label, value = (part.strip() for part in line.split("|"))
    if not label or not value:
        return None
    if label.lower() == "phase" and value == "ms":
        return None
    try:
        number = float(value)
    except ValueError:
        raise ParseDriftError(
            f"parse format drift: unparseable bench row: {line.strip()}"
        ) from None
    if not math.isfinite(number):
        raise ParseDriftError(
            f"parse format drift: non-finite bench value: {line.strip()}"
        )
    return label, number


def parse_bench_table(text: str) -> dict:
    phases: dict = {}
    for line in text.splitlines():
        row = _parse_pipe_row(line)
        if row is not None:
            phases[row[0]] = row[1]
    return phases


def parse_bench_table_multiset(text: str) -> dict:
    """Same row grammar as parse_bench_table, but EVERY sample per label
    is kept (label -> list of values). The stage5-memory bench emits
    `stage5 peak RSS` twice per scale in suite runs (curve child + the
    ratio child's re-emitted operand): two honest samples of one
    quantity, never parse drift there (RATIFIED-era consequence, T0a
    round-2 reviewer finding 3 relay)."""
    rows: dict = {}
    for line in text.splitlines():
        row = _parse_pipe_row(line)
        if row is not None:
            rows.setdefault(row[0], []).append(row[1])
    return rows


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


def find_missing_stage5_memory_rows(runs_output) -> list:
    if not runs_output:
        return list(DOCUMENTED_PHASES_STAGE5_MEMORY)
    return [row for row in DOCUMENTED_PHASES_STAGE5_MEMORY
            if any(row not in run for run in runs_output)]


def _stage5_memory_slot_medians(runs_output, slot_labels) -> dict:
    medians: dict = {}
    for label in slot_labels:
        samples = [sample for run in runs_output
                   for sample in run.get(label, [])]
        if samples:
            medians[label] = statistics.median(samples)
    return medians


def _divergent_operand(run, slot_medians, divergence_fraction):
    for label, slot_median in slot_medians.items():
        if slot_median <= 0.0:
            continue
        for sample in run.get(label, []):
            if abs(sample - slot_median) > divergence_fraction * slot_median:
                return label, sample, slot_median
    return None


def reject_divergent_peak_invocations(runs_output, divergence_fraction=None):
    """Apply the RATIFIED > 8% operand-rejection protocol.

    Any invocation carrying a peak-operand sample (the growth-ratio
    operands, i.e. `stage5 peak RSS` at BOTH scales) that diverges by
    more than divergence_fraction from the provisional per-slot median
    is rejected wholesale -- inflation AND deflation both count, because
    the round-2 erratum proved working-set trimming moves either operand
    either way. Returns (accepted_runs, rejections); rejections are
    loud accounting lines, never a silent exclusion."""
    if divergence_fraction is None:
        divergence_fraction = STAGE5_MEMORY_DIVERGENCE_FRACTION
    provisional_medians = _stage5_memory_slot_medians(
        runs_output, STAGE5_MEMORY_PEAK_LABELS)
    accepted: list = []
    rejections: list = []
    for run_index, run in enumerate(runs_output):
        offender = _divergent_operand(run, provisional_medians,
                                      divergence_fraction)
        if offender is None:
            accepted.append(run)
            continue
        label, sample, slot_median = offender
        rejections.append(
            f"REJECT invocation {run_index + 1}: {label} sample "
            f"{sample:.1f} MiB diverges "
            f"{(sample - slot_median) / slot_median * 100.0:+.1f}% from "
            f"slot median {slot_median:.1f} MiB (limit "
            f"{divergence_fraction * 100.0:.0f}%)")
    return accepted, rejections


def stage5_memory_medians(accepted_runs) -> dict:
    """Median per label over the accepted invocations' multisets. Unknown
    extra labels (e.g. the RATIFIED 4096 MiB rider probe row class, if a
    future bench emits it) are carried so comparison can report them --
    report, never crash."""
    observed = sorted({label for run in accepted_runs for label in run})
    extra = [label for label in observed
             if label not in DOCUMENTED_PHASES_STAGE5_MEMORY]
    medians: dict = {}
    for label in list(DOCUMENTED_PHASES_STAGE5_MEMORY) + extra:
        samples = [sample for run in accepted_runs
                   for sample in run.get(label, [])]
        if samples:
            medians[label] = statistics.median(samples)
    return medians


def stage5_memory_bar_verdict(medians: dict):
    """The RATIFIED absolute peak bar plus the growth tripwire, checked
    on the medians (baseline comparison alone cannot encode absolute
    bars). The growth ratio keeps being computed and printed but NEVER
    sets `regressed` (user decision 2026-10-06; t0c-verdict sec. 7).
    Returns (warnings, regressed)."""
    warnings: list = []
    regressed = False
    peak_full = medians.get(STAGE5_MEMORY_PEAK_FULL_LABEL)
    if peak_full is not None and peak_full > STAGE5_MEMORY_PEAK_BAR_MIB:
        regressed = True
        warnings.append(
            f"WARN {STAGE5_MEMORY_PEAK_FULL_LABEL}: {peak_full:.1f} MiB "
            f"exceeds the RATIFIED {STAGE5_MEMORY_PEAK_BAR_MIB:.0f} MiB "
            f"peak bar (N=1280)")
    growth_ratio = medians.get(STAGE5_MEMORY_RATIO_ROW_LABEL)
    if (growth_ratio is not None
            and growth_ratio > STAGE5_MEMORY_GROWTH_TRIPWIRE_RATIO):
        warnings.append(
            f"TRIPWIRE {STAGE5_MEMORY_RATIO_ROW_LABEL}: {growth_ratio:.2f} "
            f"exceeds the demoted growth gate "
            f"{STAGE5_MEMORY_GROWTH_TRIPWIRE_RATIO:.1f} - tripwire only, never "
            f"a gate (user decision 2026-10-06; the fixture is Theta(N^2) "
            f"by construction, see t0c-verdict sec. 7)")
    return warnings, regressed


def format_stage5_memory_value(row_label: str, value: float) -> str:
    if row_label == STAGE5_MEMORY_RATIO_ROW_LABEL:
        return f"{value:.2f} x"
    return f"{value:.1f} MiB"


def stage5_memory_baseline_key() -> str:
    """Memory rows live under their own key, machine-class-style, so a
    baseline can never mix MiB rows with ms rows on one machine class."""
    return machine_class() + STAGE5_MEMORY_BASELINE_KEY_SUFFIX


# Shared bench launcher: both modes shell their ignored cargo bench here.
def run_bench(runs: int, timeout_seconds: float, cwd: Path,
              cargo_args=None, row_parser=None) -> list:
    cargo_arguments = cargo_args or BENCH_CARGO_ARGS
    value_parser = row_parser or parse_bench_table
    collected = []
    for run_index in range(runs):
        try:
            finished = subprocess.run(
                cargo_arguments, cwd=str(cwd), capture_output=True,
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
        collected.append(value_parser(finished.stdout))
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


# Baseline comparison shared by both modes; the stage5-timing defaults
# (ratio 1.25, ms formatting) keep the original verdict bytes unchanged.
def compare_medians(medians: dict, baseline_phases: dict,
                    ratio: float = REGRESSION_RATIO,
                    value_formatter=None):
    warnings = []
    regressed = False
    for phase, median_value in medians.items():
        if phase not in baseline_phases:
            warnings.append(
                f"WARN untracked phase: {phase} (no baseline entry)")
            continue
        baseline_value = float(baseline_phases[phase])
        if median_value <= ratio * baseline_value:
            continue
        regressed = True
        if baseline_value > 0:
            increase = f"+{(median_value / baseline_value - 1.0) * 100.0:.0f}%"
        else:
            increase = "+inf%"
        if value_formatter is None:
            median_text = f"{median_value:.1f} ms"
            baseline_text = f"{baseline_value:.1f} ms"
        else:
            median_text = value_formatter(phase, median_value)
            baseline_text = value_formatter(phase, baseline_value)
        warnings.append(
            f"WARN {phase}: {median_text} vs baseline {baseline_text} "
            f"({increase})")
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
    runs = None
    timeout_seconds = float(DEFAULT_TIMEOUT_SECONDS)
    baseline_path = BASELINE_PATH
    update_baseline = False
    bench_mode = BENCH_MODE_TIMING
    index = 0
    while index < len(arguments):
        argument = arguments[index]
        if argument == "--bench":
            index += 1
            if index >= len(arguments):
                raise ValueError("--bench requires a value")
            bench_mode = arguments[index]
            if bench_mode not in BENCH_MODES:
                raise ValueError(
                    f"unknown --bench choice {bench_mode!r}; expected one "
                    f"of: {', '.join(BENCH_MODES)}")
        elif argument == "--runs":
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
    if runs is None:
        if bench_mode == BENCH_MODE_MEMORY:
            runs = STAGE5_MEMORY_DEFAULT_RUNS
        else:
            runs = DEFAULT_RUNS
    return (runs, timeout_seconds, baseline_path, update_baseline,
            bench_mode)


def _report_missing_phases(missing: list) -> None:
    print("ERROR parse format drift: documented phase row(s) missing from "
          "bench output (never substituting zeros):")
    for phase in missing:
        print(f"  missing documented phase: {phase}")


def print_stage5_memory_median_table(medians: dict) -> None:
    print("stage5-memory gate table -- RSS/size values in MiB, "
          "wall rows seconds, memo rows counts")
    print(f"{'row':<64} | {'median MiB':>10}")
    print("-" * 79)
    for row_label, median_value in medians.items():
        print(f"{row_label:<64} | {median_value:>10.1f}")


def _report_missing_stage5_memory_rows(missing: list) -> None:
    print("ERROR parse format drift: stage5-memory row(s) missing from "
          "bench output (never substituting zeros):")
    for row_label in missing:
        print(f"  missing stage5-memory row: {row_label}")


# Loud rejected-sample accounting: rejection lines first, then the
# accepted/total tally -- exclusions are never silent.
def _report_sample_accounting(accepted_count: int, total_count: int,
                              rejections: list) -> None:
    for rejection_line in rejections:
        print(rejection_line)
    print(f"stage5-memory sample accounting: accepted {accepted_count} of "
          f"{total_count} invocations, rejected "
          f"{total_count - accepted_count} (> "
          f"{STAGE5_MEMORY_DIVERGENCE_FRACTION * 100.0:.0f}% peak-operand "
          "divergence from per-slot median)")


# --update-baseline writes ONE class entry, merging the rest (operator
# PR-2 uses this for the stage5-memory namespace on VERIFIED-IDLE).
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


# Verdict path. Defaults (ratio 1.25, ms, no tail, no extra verdict) keep
# the stage5-timing output byte-identical; the memory mode passes its own
# ratio, formatter, tail, and the RATIFIED-bar verdict.
def _handle_compare(medians: dict, runs: int, baseline_path: Path,
                    class_key: str, ratio: float = REGRESSION_RATIO,
                    value_formatter=None, verdict_tail: str = "",
                    extra_verdict=None) -> int:
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
    warnings, regressed = compare_medians(
        medians, baseline_phases, ratio=ratio,
        value_formatter=value_formatter)
    if extra_verdict is not None:
        verdict_warnings, verdict_regressed = extra_verdict(medians)
        warnings.extend(verdict_warnings)
        regressed = regressed or verdict_regressed
    for warning in warnings:
        print(warning)
    if regressed:
        return 1
    print(f"bench compare: {runs} run(s), no regressions past "
          f"{ratio:.2f}x baseline{verdict_tail}")
    return 0


# stage5-timing gate flow -- unchanged since before the memory mode
# existed: same runner, same strict one-row-per-label contract, same
# exit codes and verdict lines.
def _run_timing_gate(runs: int, timeout_seconds: float, baseline_path: Path,
                     update_baseline: bool, repo_root: Path,
                     run_bench_impl) -> int:
    bench_runner = run_bench_impl or (
        lambda requested_runs, timeout, root: run_bench(
            requested_runs, timeout, root))
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


# stage5-memory gate flow: multiset parse -> RATIFIED rejection ->
# insufficient-samples guard -> medians -> peak bar + growth tripwire +
# baseline verdict.
def _run_stage5_memory_gate(runs: int, timeout_seconds: float,
                            baseline_path: Path, update_baseline: bool,
                            repo_root: Path, run_bench_impl) -> int:
    bench_runner = run_bench_impl or (
        lambda requested_runs, timeout, root: run_bench(
            requested_runs, timeout, root,
            cargo_args=STAGE5_MEMORY_CARGO_ARGS,
            row_parser=parse_bench_table_multiset))
    try:
        runs_output = bench_runner(runs, timeout_seconds, repo_root)
    except (BenchRunError, ParseDriftError) as run_failure:
        print(f"ERROR {run_failure}")
        return 2
    missing = find_missing_stage5_memory_rows(runs_output)
    if missing:
        _report_missing_stage5_memory_rows(missing)
        return 2
    accepted, rejections = reject_divergent_peak_invocations(runs_output)
    _report_sample_accounting(len(accepted), len(runs_output), rejections)
    if len(accepted) < STAGE5_MEMORY_MIN_ACCEPTED_INVOCATIONS:
        print(f"ERROR insufficient samples: accepted {len(accepted)} of "
              f"{len(runs_output)} invocations; the RATIFIED stage5-memory "
              "gate needs >= "
              f"{STAGE5_MEMORY_MIN_ACCEPTED_INVOCATIONS} accepted "
              "invocations and this hop never gates on fewer")
        return 2
    medians = stage5_memory_medians(accepted)
    print_stage5_memory_median_table(medians)
    class_key = stage5_memory_baseline_key()
    if update_baseline:
        return _handle_update(medians, len(accepted), baseline_path,
                              class_key)
    return _handle_compare(
        medians, len(accepted), baseline_path, class_key,
        ratio=STAGE5_MEMORY_REGRESSION_RATIO,
        value_formatter=format_stage5_memory_value,
        verdict_tail=" and the RATIFIED peak bar "
                     f"({STAGE5_MEMORY_PEAK_BAR_MIB:.0f} MiB), growth "
                     f"tripwire {STAGE5_MEMORY_GROWTH_TRIPWIRE_RATIO:.1f}",
        extra_verdict=stage5_memory_bar_verdict)


def main(argv=None, run_bench_impl=None) -> int:
    arguments = list(sys.argv[1:] if argv is None else argv)
    if "--help" in arguments or "-h" in arguments:
        print(USAGE_TEXT)
        return 0
    try:
        (runs, timeout_seconds, baseline_path, update_baseline,
         bench_mode) = _parse_arguments(arguments)
    except ValueError as usage_error:
        print(f"ERROR {usage_error}")
        return 2
    repo_root = Path(__file__).resolve().parents[1]
    if bench_mode == BENCH_MODE_MEMORY:
        return _run_stage5_memory_gate(
            runs, timeout_seconds, baseline_path, update_baseline,
            repo_root, run_bench_impl)
    return _run_timing_gate(
        runs, timeout_seconds, baseline_path, update_baseline, repo_root,
        run_bench_impl)


if __name__ == "__main__":
    sys.exit(main())
