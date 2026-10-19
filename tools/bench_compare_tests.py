#!/usr/bin/env python3
"""Unit tests for tools/bench_compare.py (stage-5 tail perf gate, E12).

Fully offline: every scenario feeds CANONICAL TEXT FIXTURES through
parse_bench_table and an injected fake bench runner -- cargo never runs.

Run directly:  python tools/bench_compare_tests.py
Or via CI:     python -m unittest discover -s tools -p "*_tests.py"
"""

import io
import json
import re
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import bench_compare

BENCH_TOOL_PATH = str(Path(__file__).resolve().parent / "bench_compare.py")

CANONICAL_MS = {
    "symbol_pass+prep (combined, <= first event)": 322.0,
    "file_pass (event span)": 89.5,
    "save_load_artifacts+cache (combined)": 1799.7,
    "ensure_load total": 2211.3,
    "get_or_build_include_graph clone (cold)": 1.2,
    "get_or_build_load hand-out (cold)": 1048.8,
    "get_or_build_include_graph clone (warm)": 1.4,
    "get_or_build_load hand-out (warm)": 1056.5,
    "generate_report+detect_cycle_groups (combined)": 5.8,
    "build_include_elements": 5.4,
    "path_to_id+file_collateral maps (combined)": 29.5,
    "OptimizationPlanner::plan() facts+closures": 2392.6,
    "conservative filter pass": 0.0,
    "build_file_justification_with_memo": 15383.9,
}

MISSING_PHASE = "build_file_justification_with_memo"


def render_bench_text(phase_values):
    lines = [
        "running 1 test",
        ("TAIL_BENCH headers=520 tus=40 reduced_symbols=2160 "
         "file_metrics=560 justification_entries=8960 plans=520"),
        f"{'phase':<48} | {'ms':>8}",
        "-" * 59,
    ]
    for phase, ms in phase_values.items():
        lines.append(f"{phase:<48} | {ms:>8.1f}")
    lines.append("test payload_tail_phase_timing ... ok")
    lines.append("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured")
    return "\n".join(lines) + "\n"


def run_bench_text_from(value_overrides=None, omit_phases=()):
    values = dict(CANONICAL_MS)
    for phase in omit_phases:
        values.pop(phase)
    values.update(value_overrides or {})
    return render_bench_text(values)


def make_fake_runner(run_texts):
    texts = [run_texts] if isinstance(run_texts, str) else list(run_texts)
    calls = {"run_requests": []}

    def fake_runner(requested_runs, timeout_seconds, repo_root):
        calls["run_requests"].append({
            "requested_runs": requested_runs,
            "timeout_seconds": timeout_seconds,
        })
        return [bench_compare.parse_bench_table(text) for text in texts]

    return fake_runner, calls


def invoke_main(arguments, run_texts):
    fake_runner, calls = make_fake_runner(run_texts)
    stream = io.StringIO()
    with redirect_stdout(stream):
        exit_code = bench_compare.main(arguments, run_bench_impl=fake_runner)
    return exit_code, stream.getvalue(), calls


def write_baseline_file(temp_dir, baseline):
    path = Path(temp_dir) / "bench_baseline.json"
    path.write_text(json.dumps(baseline, indent=2, sort_keys=True),
                    encoding="utf-8")
    return path


def baseline_from_canonical(overrides=None):
    phases = dict(CANONICAL_MS)
    phases.update(overrides or {})
    return {"phases": phases, "recorded": "2026-09-29", "runs": 3}


class ParseBenchTableTests(unittest.TestCase):
    def test_parses_all_documented_labels_with_floats(self):
        parsed = bench_compare.parse_bench_table(run_bench_text_from())
        self.assertEqual(set(parsed), set(bench_compare.DOCUMENTED_PHASES))
        self.assertEqual(parsed[MISSING_PHASE], 15383.9)
        self.assertEqual(parsed["file_pass (event span)"], 89.5)
        self.assertEqual(parsed["conservative filter pass"], 0.0)
        for value in parsed.values():
            self.assertIsInstance(value, float)

    def test_skips_header_rule_and_surrounding_noise(self):
        parsed = bench_compare.parse_bench_table(run_bench_text_from())
        self.assertNotIn("phase", parsed)
        self.assertNotIn("ms", parsed)
        self.assertEqual(len(parsed), 14)

    def test_tolerates_surrounding_cargo_noise_lines(self):
        noisy = ("   Compiling ariadne-cli v0.1.0\n"
                 + run_bench_text_from()
                 + "error: 0 warnings\n")
        parsed = bench_compare.parse_bench_table(noisy)
        self.assertEqual(len(parsed), 14)

    def test_non_numeric_value_raises_parse_drift(self):
        drifted = run_bench_text_from().replace("   89.5", "    8x.5")
        with self.assertRaises(bench_compare.ParseDriftError):
            bench_compare.parse_bench_table(drifted)


class MissingRowDetectionTests(unittest.TestCase):
    def test_missing_documented_label_is_named_and_exits_2(self):
        exit_code, output, _ = invoke_main(
            [], run_bench_text_from(omit_phases=[MISSING_PHASE]))
        self.assertEqual(exit_code, 2)
        self.assertIn("parse format drift", output)
        self.assertIn(MISSING_PHASE, output)

    def test_missing_rows_never_substitute_zero_medians(self):
        omitted = bench_compare.find_missing_documented_phases(
            [bench_compare.parse_bench_table(
                run_bench_text_from(omit_phases=[MISSING_PHASE]))])
        self.assertEqual(omitted, [MISSING_PHASE])


class MedianMathTests(unittest.TestCase):
    def _runs_from_per_run_values(self, per_run_ms_by_phase):
        runs = []
        run_count = len(next(iter(per_run_ms_by_phase.values())))
        for run_index in range(run_count):
            runs.append({phase: samples[run_index]
                         for phase, samples in per_run_ms_by_phase.items()})
        return runs

    def test_odd_run_count_takes_middle_sample(self):
        runs = self._runs_from_per_run_values({
            "file_pass (event span)": [80.0, 90.0, 1000.0]})
        medians = bench_compare.medians_per_phase(runs)
        self.assertEqual(medians["file_pass (event span)"], 90.0)

    def test_even_run_count_averages_the_middle_pair(self):
        runs = self._runs_from_per_run_values({
            "file_pass (event span)": [10.0, 20.0, 30.0, 40.0]})
        medians = bench_compare.medians_per_phase(runs)
        self.assertEqual(medians["file_pass (event span)"], 25.0)

    def test_medians_cover_every_documented_phase(self):
        runs = [bench_compare.parse_bench_table(run_bench_text_from())
                for _ in range(3)]
        medians = bench_compare.medians_per_phase(runs)
        self.assertEqual(set(medians), set(bench_compare.DOCUMENTED_PHASES))


class ThresholdDecisionTests(unittest.TestCase):
    def _baseline_arguments(self, temp_dir, baseline):
        path = write_baseline_file(temp_dir, baseline)
        return ["--baseline", str(path), "--runs", "1"]

    def test_26_percent_regression_exits_1_with_warn_line(self):
        with tempfile.TemporaryDirectory() as scratch:
            baseline = {bench_compare.machine_class(): baseline_from_canonical(
                {"file_pass (event span)": 71.0})}
            text = run_bench_text_from({"file_pass (event span)": 89.5})
            arguments = self._baseline_arguments(scratch, baseline)
            exit_code, output, _ = invoke_main(arguments, text)
            self.assertEqual(exit_code, 1)
            self.assertRegex(
                output,
                r"WARN file_pass \(event span\): 89\.5 ms vs baseline "
                r"71\.0 ms \(\+26%\)")

    def test_20_percent_wobble_exits_0_without_warn(self):
        with tempfile.TemporaryDirectory() as scratch:
            baseline = {bench_compare.machine_class(): baseline_from_canonical(
                {"file_pass (event span)": 74.6})}
            text = run_bench_text_from({"file_pass (event span)": 89.5})
            arguments = self._baseline_arguments(scratch, baseline)
            exit_code, output, _ = invoke_main(arguments, text)
            self.assertEqual(exit_code, 0)
            self.assertNotIn("WARN", output)

    def test_untracked_phase_warns_but_keeps_exit_0(self):
        with tempfile.TemporaryDirectory() as scratch:
            stored = baseline_from_canonical()
            stored["phases"].pop(MISSING_PHASE)
            path = write_baseline_file(scratch, {
                bench_compare.machine_class(): stored})
            arguments = ["--baseline", str(path), "--runs", "1"]
            exit_code, output, _ = invoke_main(arguments,
                                               run_bench_text_from())
            self.assertEqual(exit_code, 0)
            self.assertIn("WARN untracked phase", output)
            self.assertIn(MISSING_PHASE, output)


class NoBaselineClassTests(unittest.TestCase):
    def test_unknown_machine_class_exits_0_with_json_snapshot(self):
        with tempfile.TemporaryDirectory() as scratch:
            path = write_baseline_file(scratch, {
                "Linux-x86_64-UnknownCpu": baseline_from_canonical()})
            arguments = ["--baseline", str(path)]
            exit_code, output, _ = invoke_main(
                arguments, run_bench_text_from())
            self.assertEqual(exit_code, 0)
            self.assertIn("no baseline for machine class", output)
            self.assertIn(bench_compare.machine_class(), output)
            snapshot = json.loads(output[output.index("{"):])
            entry = snapshot[bench_compare.machine_class()]
            self.assertEqual(entry["phases"][MISSING_PHASE], 15383.9)
            self.assertEqual(entry["runs"], 3)
            self.assertRegex(entry["recorded"], r"^\d{4}-\d{2}-\d{2}$")


class UpdateBaselineTests(unittest.TestCase):
    def test_update_merges_without_clobbering_other_classes(self):
        with tempfile.TemporaryDirectory() as scratch:
            neighbor_key = "Linux-x86_64-UnknownCpu"
            second_neighbor = "Darwin-arm64-AppleM4"
            path = write_baseline_file(scratch, {
                neighbor_key: baseline_from_canonical(),
                second_neighbor: baseline_from_canonical(),
            })
            arguments = ["--baseline", str(path), "--update-baseline"]
            exit_code, output, _ = invoke_main(
                arguments, run_bench_text_from())
            self.assertEqual(exit_code, 0, output)
            merged = json.loads(path.read_text(encoding="utf-8"))
            self.assertEqual(merged[neighbor_key]["phases"],
                             CANONICAL_MS)
            self.assertEqual(merged[second_neighbor]["phases"],
                             CANONICAL_MS)
            entry = merged[bench_compare.machine_class()]
            self.assertEqual(entry["phases"][MISSING_PHASE], 15383.9)
            self.assertEqual(entry["runs"], 3)
            self.assertRegex(entry["recorded"], r"^\d{4}-\d{2}-\d{2}$")

    def test_update_overwrites_only_own_class_entry(self):
        with tempfile.TemporaryDirectory() as scratch:
            stale = baseline_from_canonical({"file_pass (event span)": 1.0})
            stale["recorded"] = "2020-01-01"
            path = write_baseline_file(scratch, {
                bench_compare.machine_class(): stale})
            arguments = ["--baseline", str(path), "--update-baseline"]
            exit_code, output, _ = invoke_main(
                arguments, run_bench_text_from())
            self.assertEqual(exit_code, 0, output)
            merged = json.loads(path.read_text(encoding="utf-8"))
            entry = merged[bench_compare.machine_class()]
            self.assertEqual(entry["phases"]["file_pass (event span)"], 89.5)
            self.assertNotEqual(entry["recorded"], "2020-01-01")


class RunCountTests(unittest.TestCase):
    def test_single_run_requests_one_cargo_invocation(self):
        with tempfile.TemporaryDirectory() as scratch:
            path = write_baseline_file(scratch, {
                bench_compare.machine_class(): baseline_from_canonical()})
            arguments = ["--baseline", str(path), "--runs", "1"]
            exit_code, output, calls = invoke_main(
                arguments, run_bench_text_from())
            self.assertEqual(exit_code, 0, output)
            self.assertEqual(len(calls["run_requests"]), 1)
            self.assertEqual(calls["run_requests"][0]["requested_runs"], 1)

    def test_default_runs_requests_three_invocations(self):
        run_texts = [run_bench_text_from() for _ in range(3)]
        with tempfile.TemporaryDirectory() as scratch:
            path = write_baseline_file(scratch, {
                bench_compare.machine_class(): baseline_from_canonical()})
            exit_code, output, calls = invoke_main(
                ["--baseline", str(path)], run_texts)
            self.assertEqual(exit_code, 0, output)
        self.assertEqual(calls["run_requests"][0]["requested_runs"], 3)

    def test_bench_run_error_maps_to_exit_2(self):
        def failing_runner(requested_runs, timeout_seconds, repo_root):
            raise bench_compare.BenchRunError("cargo exited 101")
        stream = io.StringIO()
        with redirect_stdout(stream):
            exit_code = bench_compare.main([], run_bench_impl=failing_runner)
        self.assertEqual(exit_code, 2)
        self.assertIn("cargo exited 101", stream.getvalue())


class CommandLineSurfaceTests(unittest.TestCase):
    def test_unknown_argument_is_usage_error_exit_2(self):
        finished = subprocess.run(
            [sys.executable, BENCH_TOOL_PATH, "--definitely-not-an-option"],
            capture_output=True, text=True, errors="replace")
        self.assertEqual(finished.returncode, 2)
        self.assertIn("unknown argument", finished.stdout)


class MachineClassTests(unittest.TestCase):
    def test_class_key_shape_is_system_machine_processor(self):
        import platform
        parts = bench_compare.machine_class().split("-")
        self.assertEqual(parts[0], platform.system())
        self.assertEqual(parts[1], platform.machine())
        self.assertGreaterEqual(len(parts), 3)

    def test_processor_sanitization_replaces_punctuation(self):
        sanitized = bench_compare.sanitize_processor(
            "Intel64 Family 6 Model 154 Stepping 3, GenuineIntel")
        self.assertEqual(
            sanitized, "Intel64_Family_6_Model_154_Stepping_3_GenuineIntel")
        self.assertEqual(bench_compare.sanitize_processor("  !! "), "cpu")


STAGE5_MEMORY_BENCH_SOURCE = (
    Path(__file__).resolve().parents[1]
    / "crates" / "cli" / "tests" / "stage5_memory_bench.rs")
STAGE5_MEMORY_SUFFIX_LABEL_PATTERN = re.compile(r'format!\("([^"]*) \{suffix\}"')
STAGE5_MEMORY_RATIO_TEMPLATE_GREP = (
    'format!("stage5 RSS growth ratio ({}, N={}->{})"')
HALF_PEAK_LABEL = "stage5 peak RSS (debug, N=640)"
FULL_PEAK_LABEL = "stage5 peak RSS (debug, N=1280)"
GROWTH_RATIO_LABEL = "stage5 RSS growth ratio (debug, N=640->1280)"


def stage5_memory_canonical_values():
    """One canonical value per documented stage5-memory row (MiB /
    seconds / counts as labelled). The peak pair (1350/2600) sits under
    the ratified 5200 MiB bar and its 1.93 quotient agrees with the 1.9
    ratio row under the 2.0 growth tripwire threshold (demoted from a
    gate by user decision 2026-10-06), so the canonical set passes every
    check; tests move single rows across a boundary. The
    bench prints every value at 1-decimal precision, so ratio rows move
    on the 0.1 grid the bench's own `print_row` uses."""
    values = {}
    for row_label in bench_compare.DOCUMENTED_PHASES_STAGE5_MEMORY:
        if row_label.startswith("stage5 peak RSS ("):
            values[row_label] = 1350.0 if row_label.endswith("N=640)") else 2600.0
        elif row_label.startswith("stage5 RSS growth ratio ("):
            values[row_label] = 1.9
        else:
            values[row_label] = 42.0
    return values


def render_stage5_memory_text(value_overrides=None, omit_rows=(),
                              strip_pipe_of=(), peak_extra_samples=None,
                              extra_rows=()):
    """Render one full-suite stage5_memory_bench stdout: every tuple row
    once (the curve children), then the growth-ratio child's two
    re-emitted peak operand rows (T0a verdict finding 1a), then any
    extra untracked rows. `peak_extra_samples` overrides a re-emitted
    operand value; the re-emit otherwise duplicates the child's row
    verbatim, as the bench does."""
    values = stage5_memory_canonical_values()
    values.update(value_overrides or {})
    reemitted = dict(peak_extra_samples or {})
    lines = ["running 8 tests",
             "STAGE5_BENCH profile=debug files=640 symbols=900 "
             "file_metrics=860",
             "-" * 79]
    for row_label in bench_compare.DOCUMENTED_PHASES_STAGE5_MEMORY:
        if row_label in omit_rows:
            continue
        if row_label in strip_pipe_of:
            lines.append(f"{row_label:<64} {values[row_label]:>12.1f}")
            continue
        lines.append(f"{row_label:<64} | {values[row_label]:>12.1f}")
    for row_label in (HALF_PEAK_LABEL, FULL_PEAK_LABEL):
        if row_label in omit_rows or row_label in strip_pipe_of:
            continue
        sample = reemitted.get(row_label, values[row_label])
        lines.append(f"{row_label:<64} | {sample:>12.1f}")
    for row_label, row_value in extra_rows:
        lines.append(f"{row_label:<64} | {row_value:>12.1f}")
    lines.append("test result: ok. 8 passed; 0 failed; 0 ignored; "
                 "0 measured")
    return "\n".join(lines) + "\n"


def memory_texts(count, per_index=None, uniform=None):
    """`count` invocation texts; `uniform` moves a row across a boundary
    in EVERY invocation (so it moves the median), `per_index` overrides
    single invocations (divergence-rejection scenarios)."""
    texts = [render_stage5_memory_text(**(uniform or {}))
             for _ in range(count)]
    for index, render_kwargs in (per_index or {}).items():
        texts[index] = render_stage5_memory_text(**render_kwargs)
    return texts


def stage5_memory_table_line(row_label, median_value):
    return f"{row_label:<64} | {median_value:>10.1f}"


def make_fake_memory_runner(run_texts):
    texts = [run_texts] if isinstance(run_texts, str) else list(run_texts)
    calls = {"run_requests": []}

    def fake_runner(requested_runs, timeout_seconds, repo_root):
        calls["run_requests"].append({
            "requested_runs": requested_runs,
            "timeout_seconds": timeout_seconds,
        })
        return [bench_compare.parse_bench_table_multiset(text)
                for text in texts]

    return fake_runner, calls


def invoke_memory_main(arguments, run_texts):
    fake_runner, calls = make_fake_memory_runner(run_texts)
    stream = io.StringIO()
    with redirect_stdout(stream):
        exit_code = bench_compare.main(
            ["--bench", "stage5-memory", *arguments],
            run_bench_impl=fake_runner)
    return exit_code, stream.getvalue(), calls


def memory_baseline_key_from_tool():
    return bench_compare.stage5_memory_baseline_key()


def stage5_memory_canonical_phases(overrides=None):
    phases = stage5_memory_canonical_values()
    phases.update(overrides or {})
    return phases


def stage5_memory_baseline_entry(overrides=None, runs=5):
    return {"phases": stage5_memory_canonical_phases(overrides),
            "recorded": "2026-10-06", "runs": runs}


def memory_baseline_with_entry(overrides=None):
    return {memory_baseline_key_from_tool():
            stage5_memory_baseline_entry(overrides)}


class Stage5MemoryParseTests(unittest.TestCase):
    def test_memory_mode_parses_every_documented_label_as_multiset(self):
        parsed = bench_compare.parse_bench_table_multiset(
            render_stage5_memory_text())
        self.assertEqual(
            set(parsed), set(bench_compare.DOCUMENTED_PHASES_STAGE5_MEMORY))
        self.assertEqual(len(parsed), 46)
        for peak_label in (HALF_PEAK_LABEL, FULL_PEAK_LABEL):
            self.assertIsInstance(parsed[peak_label], list)
            self.assertEqual(len(parsed[peak_label]), 2)
        for samples in parsed.values():
            for sample in samples:
                self.assertIsInstance(sample, float)

    def test_peak_label_duplicate_within_one_run_keeps_both_samples(self):
        parsed = bench_compare.parse_bench_table_multiset(
            render_stage5_memory_text(
                peak_extra_samples={FULL_PEAK_LABEL: 4321.0}))
        self.assertEqual(parsed[FULL_PEAK_LABEL], [2600.0, 4321.0])
        self.assertEqual(parsed[HALF_PEAK_LABEL], [1350.0, 1350.0])

    def test_default_parse_keeps_strict_last_write_wins_contract(self):
        duplicated = (run_bench_text_from()
                      + f"{MISSING_PHASE:<48} | {1.0:>8.1f}\n")
        parsed = bench_compare.parse_bench_table(duplicated)
        self.assertEqual(len(parsed), 14)
        self.assertEqual(parsed[MISSING_PHASE], 1.0)

    def test_stage5_row_missing_pipe_is_parse_format_drift_exit_2(self):
        with tempfile.TemporaryDirectory() as scratch:
            path = write_baseline_file(
                scratch, memory_baseline_with_entry())
            text = render_stage5_memory_text(strip_pipe_of=[HALF_PEAK_LABEL])
            exit_code, output, _ = invoke_memory_main(
                ["--baseline", str(path), "--runs", "5"], [text])
            self.assertEqual(exit_code, 2)
            self.assertIn("parse format drift", output)
            self.assertIn(HALF_PEAK_LABEL, output)


class Stage5MemoryTupleExactnessTests(unittest.TestCase):
    def test_tuple_equals_labels_grepped_from_bench_source(self):
        documented = bench_compare.DOCUMENTED_PHASES_STAGE5_MEMORY
        source = STAGE5_MEMORY_BENCH_SOURCE.read_text(encoding="utf-8")
        stems_from_source = {
            stem.rstrip() for stem
            in STAGE5_MEMORY_SUFFIX_LABEL_PATTERN.findall(source)}
        self.assertEqual(len(stems_from_source), 25)
        self.assertIn(STAGE5_MEMORY_RATIO_TEMPLATE_GREP, source)

        scale_rows = [row for row in documented
                      if re.search(r" \(debug, N=(640|1280)\)$", row)]
        warm_rows = [row for row in documented
                     if row.endswith(" (debug)")]
        ratio_rows = [row for row in documented
                      if row.startswith("stage5 RSS growth ratio (")]
        self.assertEqual(len(documented), 46)
        self.assertEqual(len(scale_rows), 40)
        self.assertEqual(len(warm_rows), 5)
        self.assertEqual(ratio_rows, [GROWTH_RATIO_LABEL])

        scale_stems = {re.sub(r" \(debug, N=(640|1280)\)$", "", row)
                       for row in scale_rows}
        warm_stems = {row[:-len(" (debug)")] for row in warm_rows}
        self.assertEqual(scale_stems | warm_stems, stems_from_source)
        self.assertEqual(len(scale_stems), 20)
        for stem in scale_stems:
            both_scales = sorted(row for row in scale_rows
                                 if row.startswith(stem + " (debug, N="))
            self.assertEqual(
                both_scales,
                [f"{stem} (debug, N=1280)", f"{stem} (debug, N=640)"])
        self.assertEqual([row for row in documented if "release" in row], [])


class Stage5MemoryThresholdDecisionTests(unittest.TestCase):
    def _run_against_baseline(self, baseline, texts, arguments=()):
        with tempfile.TemporaryDirectory() as scratch:
            path = write_baseline_file(scratch, baseline)
            return invoke_memory_main(
                ["--baseline", str(path), *arguments], texts)

    def test_peak_row_at_chosen_ratio_boundary_passes_and_past_it_regresses(self):
        ratio = bench_compare.STAGE5_MEMORY_REGRESSION_RATIO
        boundary = 2600.0 * ratio
        exit_code, passes_output, _ = self._run_against_baseline(
            memory_baseline_with_entry(),
            memory_texts(5, uniform={"value_overrides":
                                     {FULL_PEAK_LABEL: boundary}}))
        self.assertEqual(exit_code, 0, passes_output)
        self.assertNotIn("WARN", passes_output)
        exit_code, output, _ = self._run_against_baseline(
            memory_baseline_with_entry(),
            memory_texts(5, uniform={"value_overrides":
                                     {FULL_PEAK_LABEL: boundary + 0.1}}))
        self.assertEqual(exit_code, 1)
        self.assertRegex(
            output,
            r"WARN stage5 peak RSS \(debug, N=1280\): \d+\.\d MiB vs "
            r"baseline 2600\.0 MiB \(\+\d+%\)")

    def test_phase_delta_row_at_ratio_boundary_passes_and_past_it_regresses(self):
        delta_label = "symbol-pass RSS delta (debug, N=1280)"
        ratio = bench_compare.STAGE5_MEMORY_REGRESSION_RATIO
        baseline_value = 100.0
        exit_code, output, _ = self._run_against_baseline(
            memory_baseline_with_entry({delta_label: baseline_value}),
            memory_texts(5, uniform={"value_overrides":
                                     {delta_label: baseline_value * ratio}}))
        self.assertEqual(exit_code, 0, output)
        self.assertNotIn("WARN", output)
        exit_code, output, _ = self._run_against_baseline(
            memory_baseline_with_entry({delta_label: baseline_value}),
            memory_texts(5, uniform={"value_overrides":
                                     {delta_label:
                                      baseline_value * ratio + 0.1}}))
        self.assertEqual(exit_code, 1)
        self.assertIn(f"WARN {delta_label}", output)

    def test_growth_ratio_row_regression_is_first_class(self):
        ratio = bench_compare.STAGE5_MEMORY_REGRESSION_RATIO
        exit_code, output, _ = self._run_against_baseline(
            memory_baseline_with_entry({GROWTH_RATIO_LABEL: 1.0}),
            memory_texts(5, uniform={"value_overrides":
                                     {GROWTH_RATIO_LABEL: 1.0 * ratio}}))
        self.assertEqual(exit_code, 0, output)
        exit_code, output, _ = self._run_against_baseline(
            memory_baseline_with_entry({GROWTH_RATIO_LABEL: 1.0}),
            memory_texts(5, uniform={"value_overrides":
                                     {GROWTH_RATIO_LABEL: 1.6}}))
        self.assertEqual(exit_code, 1)
        self.assertIn(f"WARN {GROWTH_RATIO_LABEL}", output)

    def test_rated_peak_bar_boundary_passes_and_exceeding_regresses(self):
        peak_bar = bench_compare.STAGE5_MEMORY_PEAK_BAR_MIB
        exit_code, output, _ = self._run_against_baseline(
            memory_baseline_with_entry({FULL_PEAK_LABEL: peak_bar}),
            memory_texts(5, uniform={"value_overrides":
                                     {FULL_PEAK_LABEL: peak_bar}}))
        self.assertEqual(exit_code, 0, output)
        exit_code, output, _ = self._run_against_baseline(
            memory_baseline_with_entry({FULL_PEAK_LABEL: peak_bar}),
            memory_texts(5, uniform={"value_overrides":
                                     {FULL_PEAK_LABEL: peak_bar + 0.1}}))
        self.assertEqual(exit_code, 1)
        self.assertIn("RATIFIED", output)
        self.assertIn(FULL_PEAK_LABEL, output)

    def test_growth_tripwire_boundary_passes_and_exceeding_trips_without_failing(
            self):
        """G demoted to a tripwire by user decision 2026-10-06 (M28 T4b):
        a median exactly at 2.0 passes with NO tripwire line (strict
        exceed, same strictness as the old gate); a median above 2.0
        prints the TRIPWIRE line with the tuple-identical ratio label and
        does NOT fail the gate - the exit code is driven by the baseline
        comparison only (this fixture's baseline passes at 2.1)."""
        gate = bench_compare.STAGE5_MEMORY_GROWTH_TRIPWIRE_RATIO
        exit_code, output, _ = self._run_against_baseline(
            memory_baseline_with_entry({GROWTH_RATIO_LABEL: gate / 1.4}),
            memory_texts(5, uniform={"value_overrides":
                                     {GROWTH_RATIO_LABEL: gate}}))
        self.assertEqual(exit_code, 0, output)
        self.assertNotIn("TRIPWIRE", output)
        exit_code, output, _ = self._run_against_baseline(
            memory_baseline_with_entry({GROWTH_RATIO_LABEL: gate / 1.4}),
            memory_texts(5, uniform={"value_overrides":
                                     {GROWTH_RATIO_LABEL: 2.1}}))
        self.assertEqual(exit_code, 0, output)
        self.assertIn("TRIPWIRE " + GROWTH_RATIO_LABEL, output)

    def test_growth_tripwire_fires_independently_of_baseline_presence(self):
        """The tripwire speaks even when the ratio row has no baseline
        entry (bars/tripwire fire independently of baseline values, per
        the T0b verdict semantics): exit stays 0 (baseline rules govern
        the code), the TRIPWIRE line is present."""
        phases = stage5_memory_canonical_phases()
        phases.pop(GROWTH_RATIO_LABEL)
        baseline = {memory_baseline_key_from_tool():
                    {"phases": phases, "recorded": "2026-10-06", "runs": 5}}
        exit_code, output, _ = self._run_against_baseline(
            baseline,
            memory_texts(5, uniform={"value_overrides":
                                     {GROWTH_RATIO_LABEL: 2.1}}))
        self.assertEqual(exit_code, 0, output)
        self.assertIn("TRIPWIRE " + GROWTH_RATIO_LABEL, output)


class Stage5MemorySamplingProtocolTests(unittest.TestCase):
    def _baseline_arguments(self, scratch, runs=None):
        path = write_baseline_file(scratch, memory_baseline_with_entry())
        arguments = ["--baseline", str(path)]
        if runs is not None:
            arguments += ["--runs", str(runs)]
        return arguments

    def test_operand_divergence_rejects_invocation_in_all_four_quadrants(self):
        quadrants = [
            (HALF_PEAK_LABEL, 1525.5),
            (HALF_PEAK_LABEL, 1161.0),
            (FULL_PEAK_LABEL, 2990.0),
            (FULL_PEAK_LABEL, 2236.0),
        ]
        for label, outlier_sample in quadrants:
            with self.subTest(label=label, sample=outlier_sample):
                texts = memory_texts(6, {4: {"peak_extra_samples":
                                             {label: outlier_sample}}})
                with tempfile.TemporaryDirectory() as scratch:
                    arguments = self._baseline_arguments(scratch, runs=6)
                    exit_code, output, _ = invoke_memory_main(arguments, texts)
                    self.assertEqual(exit_code, 0, output)
                    self.assertIn("REJECT invocation 5", output)
                    self.assertIn(label, output)
                    self.assertIn("accepted 5 of 6", output)
                    self.assertIn(
                        stage5_memory_table_line(
                            label, stage5_memory_canonical_values()[label]),
                        output)

    def test_operand_divergence_of_exactly_eight_percent_is_accepted(self):
        limit = bench_compare.STAGE5_MEMORY_DIVERGENCE_FRACTION
        boundary_sample = 2600.0 * (1.0 + limit)
        texts = memory_texts(6, {3: {"peak_extra_samples":
                                     {FULL_PEAK_LABEL: boundary_sample}}})
        with tempfile.TemporaryDirectory() as scratch:
            arguments = self._baseline_arguments(scratch, runs=6)
            exit_code, output, _ = invoke_memory_main(arguments, texts)
            self.assertEqual(exit_code, 0, output)
            self.assertNotIn("REJECT", output)
            self.assertIn("accepted 6 of 6", output)

    def test_fewer_than_five_accepted_invocations_exits_2(self):
        texts = memory_texts(5, {
            1: {"peak_extra_samples": {FULL_PEAK_LABEL: 2990.0}},
            3: {"peak_extra_samples": {HALF_PEAK_LABEL: 1525.5}},
        })
        with tempfile.TemporaryDirectory() as scratch:
            arguments = self._baseline_arguments(scratch)
            exit_code, output, _ = invoke_memory_main(arguments, texts)
            self.assertEqual(exit_code, 2)
            self.assertIn("insufficient samples", output)
            self.assertIn("accepted 3 of 5", output)
            self.assertNotIn("no baseline for machine class", output)
            self.assertNotIn("no regressions past", output)

    def test_requested_runs_below_five_cannot_reach_a_verdict(self):
        with tempfile.TemporaryDirectory() as scratch:
            arguments = self._baseline_arguments(scratch) + ["--runs", "3"]
            exit_code, output, calls = invoke_memory_main(
                arguments, memory_texts(3))
            self.assertEqual(exit_code, 2)
            self.assertIn("insufficient samples", output)
            self.assertEqual(calls["run_requests"][0]["requested_runs"], 3)

    def test_memory_mode_defaults_to_five_invocations(self):
        with tempfile.TemporaryDirectory() as scratch:
            arguments = self._baseline_arguments(scratch)
            exit_code, output, calls = invoke_memory_main(
                arguments, memory_texts(5))
            self.assertEqual(exit_code, 0, output)
            self.assertEqual(calls["run_requests"][0]["requested_runs"], 5)


class Stage5MemoryBaselineKeyTests(unittest.TestCase):
    def test_absent_memory_baseline_exits_0_with_hint_and_namespaced_snapshot(self):
        with tempfile.TemporaryDirectory() as scratch:
            path = write_baseline_file(scratch, {
                "Linux-x86_64-UnknownCpu": baseline_from_canonical()})
            exit_code, output, _ = invoke_memory_main(
                ["--baseline", str(path)], memory_texts(5))
            self.assertEqual(exit_code, 0)
            self.assertIn("no baseline for machine class", output)
            self.assertIn(memory_baseline_key_from_tool(), output)
            snapshot = json.loads(output[output.index("{"):])
            entry = snapshot[memory_baseline_key_from_tool()]
            self.assertEqual(entry["phases"][FULL_PEAK_LABEL], 2600.0)
            self.assertEqual(entry["runs"], 5)
            self.assertRegex(entry["recorded"], r"^\d{4}-\d{2}-\d{2}$")

    def test_memory_rows_never_read_the_ms_machine_class_entry(self):
        poisoned_ms_entry = {
            "phases": stage5_memory_canonical_phases(
                {FULL_PEAK_LABEL: 100.0}),
            "recorded": "2026-10-06", "runs": 5}
        baseline = {bench_compare.machine_class(): poisoned_ms_entry}
        baseline.update(memory_baseline_with_entry())
        with tempfile.TemporaryDirectory() as scratch:
            path = write_baseline_file(scratch, baseline)
            exit_code, output, _ = invoke_memory_main(
                ["--baseline", str(path)], memory_texts(5))
            self.assertEqual(exit_code, 0, output)
        key = memory_baseline_key_from_tool()
        self.assertNotEqual(key, bench_compare.machine_class())
        self.assertRegex(
            key, "^" + re.escape(bench_compare.machine_class())
            + r"::stage5-memory$")

    def test_update_baseline_writes_only_the_memory_namespace(self):
        with tempfile.TemporaryDirectory() as scratch:
            path = write_baseline_file(scratch, {
                bench_compare.machine_class(): baseline_from_canonical()})
            exit_code, output, _ = invoke_memory_main(
                ["--baseline", str(path), "--update-baseline"],
                memory_texts(5))
            self.assertEqual(exit_code, 0, output)
            merged = json.loads(path.read_text(encoding="utf-8"))
            self.assertEqual(merged[bench_compare.machine_class()]["phases"],
                             CANONICAL_MS)
            entry = merged[memory_baseline_key_from_tool()]
            self.assertEqual(entry["phases"][FULL_PEAK_LABEL], 2600.0)
            self.assertEqual(entry["phases"][GROWTH_RATIO_LABEL], 1.9)
            self.assertEqual(entry["runs"], 5)


class Stage5MemoryModeIsolationTests(unittest.TestCase):
    def test_default_mode_never_gates_on_memory_rows(self):
        exit_code, output, _ = invoke_main(
            [], render_stage5_memory_text())
        self.assertEqual(exit_code, 2)
        self.assertIn("parse format drift", output)
        self.assertIn("symbol_pass+prep (combined, <= first event)", output)
        self.assertNotIn("median ms", output)

    def test_memory_mode_never_gates_on_ms_rows(self):
        exit_code, output, _ = invoke_memory_main([], run_bench_text_from())
        self.assertEqual(exit_code, 2)
        self.assertIn("parse format drift", output)
        self.assertIn(FULL_PEAK_LABEL, output)
        self.assertNotIn("no regressions past", output)

    def test_unknown_probe_row_is_reported_not_fatal(self):
        probe_row = ("save-span transient probe (debug, N=1280)", 4096.0)
        texts = memory_texts(5, {0: {"extra_rows": [probe_row]}})
        with tempfile.TemporaryDirectory() as scratch:
            path = write_baseline_file(
                scratch, memory_baseline_with_entry())
            exit_code, output, _ = invoke_memory_main(
                ["--baseline", str(path)], texts)
            self.assertEqual(exit_code, 0, output)
            self.assertIn("WARN untracked phase", output)
            self.assertIn(probe_row[0], output)


class BenchChoiceSurfaceTests(unittest.TestCase):
    def test_help_exits_zero_and_lists_both_bench_choices(self):
        finished = subprocess.run(
            [sys.executable, BENCH_TOOL_PATH, "--help"],
            capture_output=True, text=True, errors="replace")
        self.assertEqual(finished.returncode, 0)
        self.assertIn("--bench {stage5-timing,stage5-memory}",
                      finished.stdout)

    def test_explicit_timing_choice_behaves_as_the_default_mode(self):
        with tempfile.TemporaryDirectory() as scratch:
            path = write_baseline_file(scratch, {
                bench_compare.machine_class(): baseline_from_canonical()})
            fake_runner, _ = make_fake_runner(run_bench_text_from())
            stream = io.StringIO()
            with redirect_stdout(stream):
                exit_code = bench_compare.main(
                    ["--bench", "stage5-timing", "--baseline", str(path)],
                    run_bench_impl=fake_runner)
            self.assertEqual(exit_code, 0, stream.getvalue())
            self.assertIn("no regressions past 1.25x baseline",
                          stream.getvalue())

    def test_unknown_bench_choice_is_usage_error_exit_2(self):
        finished = subprocess.run(
            [sys.executable, BENCH_TOOL_PATH, "--bench", "boot-stages"],
            capture_output=True, text=True, errors="replace")
        self.assertEqual(finished.returncode, 2)
        self.assertIn("unknown --bench choice", finished.stdout)


class CorruptBaselineExitTwoTests(unittest.TestCase):
    """UTB-5 (round-2 audit): a baseline entry whose phase values are not
    usable numbers is an internal error -- exit 2 + ERROR line, never a
    float() traceback escaping main()."""

    def _run_against(self, baseline):
        with tempfile.TemporaryDirectory() as scratch:
            path = write_baseline_file(scratch, baseline)
            return invoke_main(["--baseline", str(path), "--runs", "1"],
                               run_bench_text_from())

    def _entry(self, phases):
        return {bench_compare.machine_class():
                {"phases": phases, "recorded": "2026-09-29", "runs": 3}}

    def test_non_numeric_baseline_phase_value_exits_2_not_traceback(self):
        broken = self._entry({"file_pass (event span)": "abc"})
        exit_code, output, _ = self._run_against(broken)
        self.assertEqual(2, exit_code)
        self.assertIn("ERROR", output)
        self.assertIn("file_pass (event span)", output)
        self.assertIn("is not a number", output)

    def test_unusable_phase_value_exits_2_even_for_untracked_phase(self):
        # The planted bogus phase is never compared against a median, yet
        # the entry is still corrupt: the whole phases object is validated,
        # not just the compared subset.
        broken = self._entry({"x": "abc"})
        exit_code, output, _ = self._run_against(broken)
        self.assertEqual(2, exit_code)
        self.assertIn("ERROR", output)

    def test_non_finite_baseline_phase_value_exits_2(self):
        broken = self._entry({"file_pass (event span)": float("inf")})
        exit_code, output, _ = self._run_against(broken)
        self.assertEqual(2, exit_code)
        self.assertIn("ERROR", output)
        self.assertIn("is not finite", output)

    def test_baseline_entry_without_phases_dict_exits_2(self):
        # Pin of the pre-existing sibling guard: entry without a "phases"
        # object was already the module's exit-2 contract.
        broken = {bench_compare.machine_class():
                  {"recorded": "2026-09-29", "runs": 3}}
        exit_code, output, _ = self._run_against(broken)
        self.assertEqual(2, exit_code)
        self.assertIn("ERROR", output)


class NonFiniteTimeoutArgumentTests(unittest.TestCase):
    """UTB-7 (round-2 audit) bench_compare half: --timeout nan/inf pass the
    old `value <= 0` screen and hand an unbounded deadline to the bench
    runner; both must be usage errors (exit 2) instead."""

    def _run_with_fake(self, timeout_text):
        fake_runner, _calls = make_fake_runner(run_bench_text_from())
        stream = io.StringIO()
        with redirect_stdout(stream):
            exit_code = bench_compare.main(
                ["--timeout", timeout_text], run_bench_impl=fake_runner)
        return exit_code, stream.getvalue()

    def test_nan_timeout_is_usage_error_exit_2(self):
        exit_code, output = self._run_with_fake("nan")
        self.assertEqual(2, exit_code)
        self.assertIn("--timeout", output)

    def test_inf_timeout_is_usage_error_exit_2(self):
        exit_code, output = self._run_with_fake("inf")
        self.assertEqual(2, exit_code)
        self.assertIn("--timeout", output)

    def test_positive_finite_timeout_still_accepted(self):
        with tempfile.TemporaryDirectory() as scratch:
            path = write_baseline_file(scratch, {
                bench_compare.machine_class(): baseline_from_canonical()})
            fake_runner, _calls = make_fake_runner(run_bench_text_from())
            stream = io.StringIO()
            with redirect_stdout(stream):
                exit_code = bench_compare.main(
                    ["--timeout", "120", "--baseline", str(path)],
                    run_bench_impl=fake_runner)
            output = stream.getvalue()
        self.assertEqual(0, exit_code, output)

    def test_negative_timeout_still_rejected(self):
        exit_code, output = self._run_with_fake("-5")
        self.assertEqual(2, exit_code)
        self.assertIn("--timeout", output)


if __name__ == "__main__":
    unittest.main()
