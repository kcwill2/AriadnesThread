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
    "generate_report+detect_cycles (combined)": 5.8,
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


if __name__ == "__main__":
    unittest.main()
