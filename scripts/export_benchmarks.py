#!/usr/bin/env python3
"""Aggregate Criterion benchmark results into a single JSON summary.

Usage:
    python scripts/export_benchmarks.py <criterion_dir> <output_file>

The script looks for `new/estimates.json` files beneath the Criterion
output directory produced by `cargo bench` and emits a consolidated JSON
object keyed by benchmark name. Each entry exposes the mean, median,
standard deviation, and 95% confidence interval bounds so CI or tooling
can detect performance regressions deterministically.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Dict, Any


def collect_estimates(root: Path) -> Dict[str, Any]:
    if not root.exists():
        raise FileNotFoundError(f"Criterion directory not found: {root}")

    results: Dict[str, Any] = {}
    for estimates_path in root.glob("**/new/estimates.json"):
        try:
            bench_name = estimates_path.parent.parent.name
            data = json.loads(estimates_path.read_text())
        except json.JSONDecodeError as exc:  # pragma: no cover
            raise RuntimeError(f"Failed to parse {estimates_path}: {exc}") from exc

        results[bench_name] = {
            "mean": data["mean"]["point_estimate"],
            "mean_ci_lower": data["mean"]["confidence_interval"]["lower_bound"],
            "mean_ci_upper": data["mean"]["confidence_interval"]["upper_bound"],
            "median": data["median"]["point_estimate"],
            "std_dev": data["std_dev"]["point_estimate"],
            "unit": data.get("unit", "ns"),
        }

    if not results:
        raise RuntimeError(
            f"No Criterion estimates found under {root}. Ensure `cargo bench` has been executed."
        )

    return results


def main() -> None:
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        sys.exit(2)

    criterion_dir = Path(sys.argv[1]).resolve()
    output_file = Path(sys.argv[2]).resolve()

    results = collect_estimates(criterion_dir)
    output_file.parent.mkdir(parents=True, exist_ok=True)
    output_file.write_text(json.dumps(results, indent=2, sort_keys=True) + "\n")

    print(f"Wrote benchmark summary for {len(results)} benchmarks to {output_file}")


if __name__ == "__main__":
    main()
