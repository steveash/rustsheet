"""T-3007: write the bratio branch fixture of SPEC-0025's "Test fixtures" from T-2464's BETA.DIST cells.

Every cumulative four-argument BETA.DIST cell of docs/research/t2464/cells.tsv (excel-graph values) becomes a row
`a b x y excel branch swapped`: a, b, x, y = fl(1 - x) and Excel's value as 16-digit hex f64 bits, then the branch
r2-bratio's model `bratio_excel.py` `kernel` names for the point (its own name, spaces as underscores; the Rust test
maps it to `bratio::Branch`) and 1 when bratio swapped. Branches with more than CAP rows keep every k-th row in (a, b, x) order.

Run from the repository root (the model needs numpy and mpmath; its GAMMALN placeholder does not change a branch):
    uv run --no-project --with mpmath --with numpy python docs/research/t3007/make_bratio_fixture.py
"""

from __future__ import annotations

import csv
import struct
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "docs/research/t2464/r2-bratio"))
import bratio_excel  # noqa: E402  (importable only once sys.path has r2-bratio)

CELLS = ROOT / "docs/research/t2464/cells.tsv"
OUT = ROOT / "crates/rustsheet-engine/src/test/fixtures/bratio_beta_dist.fixture"
CAP = 400

def hexbits(v: float) -> str:
    return struct.pack(">d", v).hex()


def main() -> None:
    rows: dict[str, list[tuple[float, float, float, float, float, int]]] = defaultdict(list)
    seen = set()
    with CELLS.open(newline="") as f:
        for cell in csv.DictReader(f, delimiter="\t"):
            args = cell["args"].split("|")
            if cell["fn"] != "BETA.DIST" or len(args) != 4 or args[3] != "True":
                continue
            try:
                x, a, b, excel = float(args[0]), float(args[1]), float(args[2]), float(cell["excel"])
            except ValueError:
                continue
            if not 0.0 <= x <= 1.0 or (a, b, x) in seen:
                continue
            seen.add((a, b, x))
            y = 1.0 - x
            _, _, name = bratio_excel.kernel(a, b, x, y)
            swapped = name.endswith("/sw")
            name = name.removesuffix("/sw")
            if name.startswith("closed"):
                continue
            rows[name.replace(" ", "_")].append((a, b, x, y, excel, int(swapped)))
    lines = [
        "# T-3007 bratio branch fixture (SPEC-0025 \"Test fixtures\"): a b x y excel (f64 bits) branch swapped.",
        "# Cumulative BETA.DIST cells of docs/research/t2464/cells.tsv (excel-graph), y = fl(1 - x); branch and",
        "# swap as r2-bratio's bratio_excel.py kernel names them. Written by docs/research/t3007/make_bratio_fixture.py.",
    ]
    for branch in sorted(rows):
        points = sorted(rows[branch])
        step = max(1, -(-len(points) // CAP))
        for a, b, x, y, excel, swapped in points[::step]:
            lines.append(f"{hexbits(a)} {hexbits(b)} {hexbits(x)} {hexbits(y)} {hexbits(excel)} {branch} {swapped}")
        print(f"{branch}: {len(points)} cells, {len(points[::step])} rows")
    OUT.write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()
