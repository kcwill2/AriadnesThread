#!/usr/bin/env python3
"""M28 T0c measurement helper: env-setter, disk witness, ratified medians law.

Why this file exists: chained `$env:` shell forms hit the global catch-all
deny rule (ledger row 61), so every bench-env setting for the T0c groups
goes through this helper, and the helper is distill-routed so the OUTER
distiller owns capture + digest (stdout stays a passthrough; the child exit
code propagates - I2 truth).

Modes:
  --files N [--caps a,b] -- <argv...>
      Set STAGE5_MEM_BENCH_FILES (and optionally ARIADNE_LOAD_MEM_CAPS) in
      the child env, run <argv> with inherited stdio, exit with the child's
      exit code.
  --disk-witness --min-free-gib F
      One line `DISK C: free <n> GiB of <total> GiB - PASS|STOP`; exit 0 iff
      free >= F GiB (precondition 1: the os error 112 history).
  --medians <capture...>
      The ratified sampling law (DP-4 review, T0b handoff sec. "RATIFIED
      gate contract"): per-label multiset parse of `label | value` rows;
      provisional per-slot medians over the peak-label pool; wholesale
      REJECT any invocation with any peak sample diverging >8% STRICT from
      the provisional slot median (binding on both operands at both scales
      - the operands ARE the peak slots, so checking every peak sample
      covers both operands symmetrically); loud rejected-sample accounting,
      never silent exclusion; then per-label medians over ACCEPTED
      invocations for peak labels and over ALL invocations for non-peak
      labels; fewer than 5 accepted prints `GROUP INCOMPLETE (K of N)` and
      exits 1 - never a verdict.

Pure ASCII, functions under 100 lines (hygiene law)."""

import math
import os
import shutil
import statistics
import subprocess
import sys

SCALE_FILES_ENV = "STAGE5_MEM_BENCH_FILES"
CAPS_ENV = "ARIADNE_LOAD_MEM_CAPS"
PEAK_LABEL_PREFIX = "stage5 peak RSS "
REJECTION_THRESHOLD_FRACTION = 0.08
MIN_ACCEPTED_INVOCATIONS = 5
BYTES_PER_GIB = 1024 ** 3


def parse_cap_token(token, raw_value):
    """One caps token: non-negative integer or 'max' (-> usize::MAX)."""
    trimmed = token.strip()
    if trimmed == "max":
        return (1 << 64) - 1
    if trimmed.isdigit():
        return int(trimmed)
    raise ValueError(
        "%s token %r invalid (want non-negative integer or 'max'), raw %r"
        % (CAPS_ENV, token, raw_value)
    )


def caps_shape_is_valid(raw_value):
    """True iff the value is `a,b` with both tokens parseable."""
    tokens = raw_value.split(",")
    if len(tokens) != 2:
        return False
    try:
        for token in tokens:
            parse_cap_token(token, raw_value)
    except ValueError:
        return False
    return True


def run_with_env(files_value, caps_value, child_argv):
    """Child inherits this process's env + stdio; exit code propagates."""
    child_env = os.environ.copy()
    if files_value is not None:
        child_env[SCALE_FILES_ENV] = str(files_value)
    if caps_value is not None:
        child_env[CAPS_ENV] = caps_value
    finished = subprocess.run(child_argv, env=child_env)
    return finished.returncode


def report_disk_witness(min_free_gib):
    """One PASS/STOP line for the system drive; 0 iff free >= min_free_gib."""
    if os.name == "nt":
        probe_root = os.environ.get("SystemDrive", "C:") + "\\"
    else:
        probe_root = "/"
    usage = shutil.disk_usage(probe_root)
    free_gib = usage.free // BYTES_PER_GIB
    total_gib = usage.total // BYTES_PER_GIB
    sufficient = usage.free >= min_free_gib * BYTES_PER_GIB
    verdict = "PASS" if sufficient else "STOP"
    drive_label = probe_root[:1].upper() + ":"
    print("DISK %s free %d GiB of %d GiB - %s"
          % (drive_label, free_gib, total_gib, verdict))
    return 0 if sufficient else 1


def strip_harness_prefix(label):
    """Normalize cargo's --nocapture interleaving: the harness prints
    `test <name> ... ` without a newline, so a test's FIRST printed table
    row arrives prefixed (the growth-ratio test's re-emitted operand
    rows). The bench labels never contain ` ... `, so the prefix is safe
    to peel."""
    if label.startswith("test "):
        head, separator, tail = label.partition(" ... ")
        test_name = head[len("test "):]
        if separator and test_name and " " not in test_name:
            return tail.strip()
    return label


def parse_table_rows(capture_path):
    """`label | value` rows of one capture as [(label, value)] (multiset)."""
    rows = []
    with open(capture_path, "r", encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if line.count("|") != 1:
                continue
            label, _, raw_value = line.partition("|")
            label = strip_harness_prefix(label.strip())
            try:
                value = float(raw_value.strip())
            except ValueError:
                continue
            if not label or not math.isfinite(value):
                continue
            rows.append((label, value))
    return rows


def provisional_slot_medians(peak_samples_per_invocation):
    """Median of every peak sample per slot label, over ALL invocations."""
    sample_pool = {}
    for samples in peak_samples_per_invocation:
        for label, value in samples:
            sample_pool.setdefault(label, []).append(value)
    return {label: statistics.median(values)
            for label, values in sample_pool.items()}


def rejection_reasons(peak_samples, slot_medians):
    """Every >8% strict divergence of this invocation's peak samples."""
    reasons = []
    for label, value in peak_samples:
        median = slot_medians[label]
        divergence = abs(value - median) / median
        if divergence > REJECTION_THRESHOLD_FRACTION:
            reasons.append("%s sample %.1f vs slot median %.1f = %.1f%%"
                           % (label, value, median, divergence * 100.0))
    return reasons


def values_per_invocation(all_rows, label):
    """Per-invocation sample lists (multiset preserved) for one label."""
    return [[(row_label, value) for row_label, value in rows
             if row_label == label]
            for rows in all_rows]


def print_label_median_rows(all_rows, accepted_indices, label):
    """Per-invocation values + the law-conform median line for one label."""
    is_peak = label.startswith(PEAK_LABEL_PREFIX)
    pooled_values = []
    for invocation_index, samples in enumerate(values_per_invocation(
            all_rows, label), start=1):
        if not samples:
            continue
        if is_peak and invocation_index not in accepted_indices:
            continue
        for _, value in samples:
            pooled_values.append(value)
            print("INV %d %s | %.1f" % (invocation_index, label, value))
    if not pooled_values:
        return
    median_basis = "accepted" if is_peak else "all"
    print("MEDIAN(%s) %s | %.1f"
          % (median_basis, label, statistics.median(pooled_values)))


def run_medians(capture_paths):
    """The ratified law over listed captures; 0 iff >=5 accepted, else 1."""
    all_rows = [parse_table_rows(path) for path in capture_paths]
    empty_captures = [path for path, rows in zip(capture_paths, all_rows)
                      if not rows]
    if empty_captures:
        print("usage error: no table rows parsed from: "
              + ", ".join(empty_captures))
        return 2
    peak_samples_per_invocation = [
        [(label, value) for label, value in rows
         if label.startswith(PEAK_LABEL_PREFIX)]
        for rows in all_rows
    ]
    slot_medians = provisional_slot_medians(peak_samples_per_invocation)
    accepted_indices = []
    for invocation_index, samples in enumerate(
            peak_samples_per_invocation, start=1):
        reasons = rejection_reasons(samples, slot_medians)
        if reasons:
            print("REJECT invocation %d: %s"
                  % (invocation_index, "; ".join(reasons)))
        else:
            accepted_indices.append(invocation_index)
    for label in sorted({label for rows in all_rows for label, _ in rows}):
        print_label_median_rows(all_rows, accepted_indices, label)
    print("accepted %d of %d" % (len(accepted_indices), len(all_rows)))
    if len(accepted_indices) < MIN_ACCEPTED_INVOCATIONS:
        print("GROUP INCOMPLETE (%d of %d)"
              % (len(accepted_indices), len(all_rows)))
        return 1
    return 0


def parse_options(argv):
    """Returns (mode, options, remainder); mode in run/disk/medians/None."""
    options = {"files": None, "caps": None, "min_free_gib": None}
    mode = None
    index = 0
    while index < len(argv):
        argument = argv[index]
        index += 1
        if argument == "--medians":
            return "medians", options, list(argv[index:])
        if argument == "--disk-witness":
            mode = "disk"
        elif argument == "--min-free-gib":
            options["min_free_gib"] = int(argv[index])
            index += 1
        elif argument == "--files":
            options["files"] = int(argv[index])
            index += 1
            mode = "run"
        elif argument == "--caps":
            options["caps"] = argv[index]
            index += 1
            mode = "run"
        elif argument == "--":
            return mode or "run", options, list(argv[index:])
        else:
            raise ValueError("unknown helper option before --: %r" % argument)
    return mode, options, []


def main(argv=None):
    arguments = list(sys.argv[1:] if argv is None else argv)
    try:
        mode, options, remainder = parse_options(arguments)
    except ValueError as bad_usage:
        print("usage error: %s" % bad_usage)
        return 2
    if mode == "disk":
        if options["min_free_gib"] is None:
            print("usage error: --disk-witness needs --min-free-gib F")
            return 2
        return report_disk_witness(options["min_free_gib"])
    if mode == "medians":
        if not remainder:
            print("usage error: --medians needs at least one capture path")
            return 2
        return run_medians(remainder)
    if mode == "run":
        if not remainder:
            print("usage error: run mode needs `-- <argv...>`")
            return 2
        if options["caps"] is not None and not caps_shape_is_valid(
                options["caps"]):
            print("usage error: %s must be 'a,b' with each token a "
                  "non-negative integer or 'max', got %r"
                  % (CAPS_ENV, options["caps"]))
            return 2
        return run_with_env(options["files"], options["caps"], remainder)
    print("usage error: no mode given (--files/--caps -- argv, "
          "--disk-witness, or --medians)")
    return 2


if __name__ == "__main__":
    sys.exit(main())
