#!/usr/bin/env python3
"""Compare Criterion benchmark summaries against a baseline.

Usage:
    python scripts/compare_benchmarks.py <baseline_json> <summary_json> [--max-regression 0.1]

The script exits with a non-zero status code if any monitored benchmark
regresses beyond the provided percentage threshold or if expected benchmarks
are missing from the latest run.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Dict, Tuple, List


def load_summary(path: Path) -> Dict[str, Any]:
    if not path.exists():
        raise FileNotFoundError(f"Benchmark summary not found: {path}")
    try:
        return json.loads(path.read_text())
    except json.JSONDecodeError as exc:  # pragma: no cover
        raise RuntimeError(f"Failed to parse JSON from {path}: {exc}") from exc


def format_ns(value: float) -> str:
    return f"{value:.3f} ns"


def compare(
    baseline: Dict[str, Any],
    current: Dict[str, Any],
    max_regression: float,
    absolute_slack: float,
) -> Tuple[List[str], List[Tuple[str, float, float]]]:
    missing = []
    regressions: List[Tuple[str, float, float]] = []

    for bench, baseline_stats in baseline.items():
        if bench not in current:
            missing.append(bench)
            continue

        try:
            baseline_mean = float(baseline_stats["mean"])
            current_mean = float(current[bench]["mean"])
        except (KeyError, TypeError, ValueError) as exc:
            raise RuntimeError(
                f"Benchmark statistics for '{bench}' are malformed: {exc}"
            ) from exc

        if baseline_mean <= 0.0:
            threshold = absolute_slack
        else:
            threshold = baseline_mean * (1.0 + max_regression)

        if current_mean > max(threshold, absolute_slack):
            regressions.append((bench, baseline_mean, current_mean))

    return missing, regressions


def main(argv: List[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path, help="Path to the baseline JSON file")
    parser.add_argument("current", type=Path, help="Path to the current benchmark summary JSON file")
    parser.add_argument(
        "--max-regression",
        type=float,
        default=0.1,
        help="Maximum allowed relative regression (e.g. 0.1 for 10%%)",
    )
    parser.add_argument(
        "--absolute-slack",
        type=float,
        default=1.0,
        help="Absolute slack in nanoseconds for zero-baseline benchmarks",
    )
    args = parser.parse_args(argv)

    baseline = load_summary(args.baseline)
    current = load_summary(args.current)

    missing, regressions = compare(
        baseline,
        current,
        max_regression=args.max_regression,
        absolute_slack=args.absolute_slack,
    )

    new_benchmarks = sorted(set(current.keys()) - set(baseline.keys()))

    if new_benchmarks:
        print("ℹ️  Detected new benchmarks not present in baseline:")
        for bench in new_benchmarks:
            print(f"   • {bench}")
        print()

    if not missing and not regressions:
        print(
            "✅ Benchmarks are within the allowed regression threshold "
            f"({args.max_regression * 100:.1f}% max)."
        )
        return 0

    if missing:
        print("❌ Missing benchmarks in current run:")
        for bench in missing:
            print(f"   • {bench}")
        print()

    if regressions:
        print("❌ Benchmarks exceeding allowed regression:")
        for bench, baseline_mean, current_mean in regressions:
            delta = current_mean - baseline_mean
            delta_pct = (delta / baseline_mean * 100.0) if baseline_mean else float("inf")
            print(
                f"   • {bench}: baseline {format_ns(baseline_mean)}, "
                f"current {format_ns(current_mean)} (+{delta_pct:.2f}%)"
            )

    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
