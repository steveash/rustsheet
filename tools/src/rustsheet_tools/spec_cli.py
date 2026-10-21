"""`rustsheet-spec`: edits to case files that need the case builder's knowledge (T-2084)."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

from rustsheet_tools.pivot import rendered_cells


def bake_pivots(spec: dict[str, Any]) -> bool:
    """Sets every pivot's `rendered` to the builder's layout; returns whether anything changed."""
    changed = False
    for sheet in spec["sheets"]:
        for pivot in sheet.get("pivot_tables", ()):
            layout = rendered_cells(spec, sheet, pivot)
            if pivot.get("rendered") != layout:
                pivot["rendered"] = layout
                changed = True
    return changed


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="rustsheet-spec", description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    bake = commands.add_parser(
        "bake-pivots", help="fill each pivot's `rendered` cells from the builder's layout"
    )
    bake.add_argument("cases", nargs="+", type=Path, help="case files (`*.case.json`)")
    args = parser.parse_args(argv)
    for path in args.cases:
        spec = json.loads(path.read_text())
        if bake_pivots(spec):
            path.write_text(json.dumps(spec, indent=2, ensure_ascii=False) + "\n")
            sys.stdout.write(f"baked {path}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
