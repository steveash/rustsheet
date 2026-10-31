"""T-3347: score the IRR conventional stage on its train cells and its R4 hold-out (decision 0188 R4.1).

Usage (from the repo root, after `cargo build --release -p rustsheet_eval`):

    python3 docs/research/t3347_irr_holdout_score.py

For each of the three cases (train: the probe and the Gate A sweep; hold-out: the post-freeze
sweep) it reads the `.roots.tsv` written by t3347_irr_conventional_sweep.py (Excel's value and the
50-digit root) and the engine's value from `rustsheet-eval show --all`, and prints per stratum:
bit-exact, within 4 ulp, within 15 significant digits of Excel, type mismatches (a number against an
error, or two different errors), and Gate A points outside both bounds (15 digits of Excel or 1e-9
relative of the root), for the engine and for the correctly rounded root.
"""

from __future__ import annotations

import math
import re
import subprocess
import sys
from collections import defaultdict
from decimal import Decimal
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CASES = (
    ("train", "probes/t3347-irr-conventional-num.case.json"),
    ("train", "probes-numeric/t3347-irr-conventional-sweep.case.json"),
    ("hold-out", "probes-numeric/t3347-irr-conventional-holdout.case.json"),
)
CELL = re.compile(r"^\s*(?:\w+!)?(A\d+)\b")
ACTUAL = re.compile(r"^\s*actual\s*:\s*(\S+)")


def engine_values(case: str) -> dict[str, str]:
    out = subprocess.run(
        [str(ROOT / "target/release/rustsheet-eval"), "--root", str(ROOT), "show", case, "--all"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    values: dict[str, str] = {}
    cell = None
    for line in out.splitlines()[1:]:
        if m := ACTUAL.match(line):
            if cell:
                values[cell] = m.group(1)
        elif m := CELL.match(line):
            cell = m.group(1)
    return values


def as_number(text: str) -> float | None:
    try:
        return float(text)
    except ValueError:
        return None


def ulps(a: float, b: float) -> float:
    return abs(a - b) / math.ulp(max(abs(a), abs(b))) if a != b else 0.0


def within_15(a: float, b: float) -> bool:
    return a == b or abs(a - b) <= 5e-15 * max(abs(a), abs(b))


def score(excel: str, got: str, root: Decimal) -> dict[str, int]:
    x, g = as_number(excel), as_number(got)
    if x is None or g is None:
        same = x is None and g is None and excel == got
        return {"bit": same, "ulp4": same, "d15": same, "type": not same, "outside": not same}
    in_math = abs(Decimal(g) - root) <= Decimal("1e-9") * abs(root) or abs(Decimal(g) - root) <= Decimal(
        "1e-300"
    )
    return {
        "bit": g == x,
        "ulp4": ulps(g, x) <= 4,
        "d15": within_15(g, x),
        "type": False,
        "outside": not (within_15(g, x) or in_math),
    }


def main() -> None:
    totals: dict[tuple[str, str, str], dict[str, int]] = defaultdict(lambda: defaultdict(int))
    for split, case in CASES:
        roots = (ROOT / "eval/suites" / case.replace("/", "/cases/", 1).replace(".case.json", ".roots.tsv")).read_text().splitlines()[1:]
        engine = engine_values(case)
        for row in roots:
            cell, stratum, excel, root_text = row.split("\t")
            root = Decimal(root_text)
            rounded = repr(float(root))
            for who, got in (("engine", engine.get(cell, "?")), ("rounded", rounded)):
                for key in ((split, who, stratum), (split, who, "ALL")):
                    t = totals[key]
                    t["n"] += 1
                    for k, v in score(excel, got, root).items():
                        t[k] += int(v)
                if who == "engine" and as_number(excel) is None and excel != got:
                    print(f"  {split} {case.split('/')[0]} {cell} {stratum}: Excel {excel}, engine {got}")
    print("split\tcandidate\tstratum\tn\tbit\t<=4ulp\t15dig\ttype-mismatch\toutside-both")
    for (split, who, stratum), t in sorted(totals.items()):
        print(
            f"{split}\t{who}\t{stratum}\t{t['n']}\t{t['bit']}\t{t['ulp4']}\t{t['d15']}\t{t['type']}\t{t['outside']}"
        )


if __name__ == "__main__":
    sys.exit(main())
