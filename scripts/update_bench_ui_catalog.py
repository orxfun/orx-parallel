#!/usr/bin/env python3
"""Regenerates docs/bench-ui/results/catalog.json from existing CSV files."""

from __future__ import annotations

import json
from pathlib import Path


def main() -> None:
    repo_root = Path(__file__).resolve().parents[1]
    results_dir = repo_root / "docs" / "bench-ui" / "results"
    catalog_path = results_dir / "catalog.json"

    if not results_dir.is_dir():
        raise SystemExit(f"Results directory not found: {results_dir}")

    catalog: dict[str, list[str]] = {}
    for category_dir in sorted(
        (p for p in results_dir.iterdir() if p.is_dir()), key=lambda p: p.name
    ):
        benches = sorted(
            csv_file.stem
            for csv_file in category_dir.glob("*.csv")
            if csv_file.is_file()
        )
        catalog[category_dir.name] = benches

    catalog_path.write_text(json.dumps(catalog, indent=2) + "\n", encoding="utf-8")
    print(f"Updated {catalog_path}")


if __name__ == "__main__":
    main()
