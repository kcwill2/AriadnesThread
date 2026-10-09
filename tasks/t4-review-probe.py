"""M28 T4 reviewer scratch probe - independent curve recompute.

Parses target/t4-postfix-*.txt with its OWN regexes and re-implements the
ratified sampling law standalone (no implementer helper code): >8% strict
operand divergence vs the provisional per-slot median rejects an
invocation (binding on both operands at both scales); peak-label medians
over ACCEPTED invocations; non-peak medians over ALL; <5 accepted =
protocol failure. Prints the acceptance reading and the transient.
"""
import statistics
from pathlib import Path

TARGET = Path("target")
CAPTURES = [TARGET / ("t4-postfix-%d.txt" % k) for k in range(1, 6)]

PEAK1280 = "stage5 peak RSS (release, N=1280)"
PEAK640 = "stage5 peak RSS (release, N=640)"
RATIO = "stage5 RSS growth ratio (release, N=640->1280)"
SAVE_TAIL1280 = "RSS checkpoint save tail (release, N=1280)"


def rows(path):
    out = {}
    for line in path.read_text(errors="replace").splitlines():
        if "|" not in line:
            continue
        label, _, value = line.rpartition("|")
        label = label.split("...", 1)[-1].strip()
        try:
            out.setdefault(label, []).append(float(value))
        except ValueError:
            pass
    return out


def main():
    parsed = [rows(path) for path in CAPTURES]
    for label in (PEAK1280, PEAK640):
        counts = [len(p.get(label, [])) for p in parsed]
        assert counts == [2, 2, 2, 2, 2], (label, counts)  # peak multiset
    for label in (RATIO, SAVE_TAIL1280):
        counts = [len(p.get(label, [])) for p in parsed]
        assert counts == [1, 1, 1, 1, 1], (label, counts)

    # Multiset law: two peak samples per invocation per slot. Provisional
    # slot medians over the whole pool; an invocation is rejected if ANY of
    # its four peak operands diverges >8% strictly (both operands, both
    # scales); accepted slot medians over all samples of accepted
    # invocations (same result as per-invocation medians at this spread).
    pool1280_all = [v for p in parsed for v in p[PEAK1280]]
    pool640_all = [v for p in parsed for v in p[PEAK640]]
    med1280 = statistics.median(pool1280_all)
    med640 = statistics.median(pool640_all)

    accepted = []
    for index, path_rows in enumerate(parsed, start=1):
        worst = 0.0
        for value in path_rows[PEAK1280]:
            worst = max(worst, abs(value - med1280) / med1280)
        for value in path_rows[PEAK640]:
            worst = max(worst, abs(value - med640) / med640)
        if worst * 100.0 > 8.0:
            print("REJECT invocation %d: %.1f%% > 8%%" % (index, worst * 100.0))
        else:
            accepted.append(index)
    print("accepted %d of %d (independent law, multiset operands)"
          % (len(accepted), len(parsed)))
    assert len(accepted) == 5, "protocol floor: >=5 accepted"

    acc1280 = [v for i in accepted for v in parsed[i - 1][PEAK1280]]
    acc640 = [v for i in accepted for v in parsed[i - 1][PEAK640]]
    peak1280_median = statistics.median(acc1280)
    peak640_median = statistics.median(acc640)
    ratio_median = statistics.median([statistics.median(p[RATIO]) for p in parsed])
    save_median = statistics.median([p[SAVE_TAIL1280][0] for p in parsed])

    print("independent MEDIAN(accepted) peak N=1280 = %.1f" % peak1280_median)
    print("independent MEDIAN(accepted) peak N=640  = %.1f" % peak640_median)
    print("independent MEDIAN(ratios)               = %.1f" % ratio_median)
    print("independent MEDIAN(all) save tail@1280   = %.1f" % save_median)
    print("derived save-span transient              = %.1f"
          % (peak1280_median - save_median))
    print("acceptance: %.1f <= 5200.0 -> %s"
          % (peak1280_median, peak1280_median <= 5200.0))
    print("peak values 1280:", pool1280)
    print("peak values 640:", pool640)


if __name__ == "__main__":
    main()
