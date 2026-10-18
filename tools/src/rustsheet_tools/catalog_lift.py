"""Check eval/catalog/functions.yaml against the engine's SPEC-0011 argument-kind table (T-1776).

The table comes from eval/catalog/lift-kinds.json, which `rustsheet-eval lift-kinds --write`
writes and a Rust test keeps current, so this module never builds the engine. `check` applies the
spec's mapping rule; `uv run rustsheet-catalog-lift` prints its findings and fails on any, and
`--write` first adds the `lift: arrays` keys the table derives.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

from rustsheet_tools.gen.generator import Catalog

REPO = Path(__file__).resolve().parents[3]
CATALOG = REPO / "eval" / "catalog" / "functions.yaml"
TABLE = CATALOG.with_name("lift-kinds.json")

WHOLE_MODES = frozenset({"array", "aggregate", "aggregate_a", "range", "criteria", "lambda"})
# SPEC-0011 "The argument-kind table": the catalog modes each kind may carry, and whether the
# parameter carries `lift: arrays` (it lifts over arrays and refuses a multi-cell range).
RULE: dict[str, tuple[frozenset[str], bool]] = {
    "Lift": (frozenset({"value"}), False),
    "LiftArrays": (frozenset({"value"}), True),
    "LiftArraysRangeWhole": (frozenset({"criteria", "range"}), True),
    "Whole": (WHOLE_MODES, False),
    "WholeByCaller": (WHOLE_MODES, False),
}


def load_table(path: Path = TABLE) -> list[dict[str, Any]]:
    table: list[dict[str, Any]] = json.loads(path.read_text())
    return table


def check(functions: list[dict[str, Any]], table: list[dict[str, Any]]) -> list[str]:
    """Every way `functions` (catalog entries) disagrees with `table`, one message each."""
    by_name = {str(f["name"]): f for f in functions}
    problems: list[str] = []
    for row in table:
        name = row["name"]
        entry = by_name.get(name)
        if entry is None or "params" not in entry:
            problems.append(f"{name}: in the lifting table but has no catalog params")
            continue
        params: list[dict[str, Any]] = entry["params"]
        kinds: list[str] = row["params"]
        if row["repeat"] is None and len(params) != len(kinds):
            problems.append(f"{name}: catalog has {len(params)} params, the table {len(kinds)}")
        for param, kind in zip(params, kinds, strict=False):
            modes, lifts_arrays = RULE[kind]
            label = f"{name}({param['name']})"
            if param.get("mode", "value") not in modes:
                problems.append(f"{label}: mode {param.get('mode')} does not fit {kind}")
            if (param.get("lift") == "arrays") != lifts_arrays:
                want = "needs" if lifts_arrays else "must not have"
                problems.append(f"{label}: {kind} {want} `lift: arrays`")
    lifted = {row["name"] for row in table}
    for name, entry in by_name.items():
        if name not in lifted and any(p.get("lift") for p in entry.get("params", [])):
            problems.append(f"{name}: `lift:` set but the function is not in the lifting table")
    return problems


def write(text: str, table: list[dict[str, Any]]) -> str:
    """`text` (the catalog YAML) with `lift: arrays` after `mode: value` on every parameter whose
    kind lifts over arrays only. Line-based, so comments and flow layout survive."""
    wanted = {
        (row["name"], index) for row in table for index, kind in enumerate(row["params"]) if RULE[kind][1]
    }
    out: list[str] = []
    function, index = "", -1
    for line in text.splitlines(keepends=True):
        stripped = line.strip()
        if line.startswith("  - name: "):
            function, index = stripped.removeprefix("- name: "), -1
        elif line.startswith(("      - { name: ", "    params: [{ name: ")):
            # A one-parameter function writes its list inline: `params: [{ name: number, ... }]`.
            index += 1
            if (function, index) in wanted and "lift:" not in line:
                out.append(line.replace("mode: value,", "mode: value, lift: arrays,", 1))
                continue
        out.append(line)
    return "".join(out)


def main() -> int:
    if "--write" in sys.argv[1:]:
        CATALOG.write_text(write(CATALOG.read_text(), load_table()))
    problems = check(Catalog.load(CATALOG).functions, load_table())
    for problem in problems:
        print(problem)
    print(f"{len(problems)} disagreement(s) between {CATALOG.name} and {TABLE.name}")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
