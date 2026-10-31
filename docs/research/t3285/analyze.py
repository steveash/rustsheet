"""T-3285: score the fresh sweep (mksweep.py) against Excel and the 50-digit math, without fitting.

  --save LABEL [--report PATH]
                write the engine's value per cell to sweep-LABEL.json beside this script: from `rustsheet-eval show
                --all` with ./target/release/rustsheet-eval, or from a `rustsheet-eval run` report.json (it lists
                failing cells; a cell it does not list is Excel's value). main and head were taken from the T-3285
                main-vs-HEAD runs' reports;
  (default)     for each saved label and for the correctly rounded math: cells equal to Excel's double, cells within
                15 significant digits of Excel (decision 0083), and Gate A's envelope (decision 0188 R2: within 1e-9
                relative of the math, absolute floor 1e-300, or within 15 digits of Excel), split at x = 30.
Usage (repo root): uv run --with mpmath python docs/research/t3285/analyze.py [--save LABEL]"""

from __future__ import annotations

import json
import math
import re
import struct
import subprocess
import sys
from pathlib import Path

import mpmath

HERE = Path(__file__).parent
CASE = "probes-numeric/t3285-chisq-dist-rt-half-integer-sweep.case.json"
CELL = re.compile(r"^(Calc!A\d+)\s")
VAL = re.compile(r"^\s+(formula|expected|actual)\s*:\s*(\S+)")
EPS = 1e-9
FLOOR = 1e-300


def show() -> dict[str, dict[str, str]]:
    out = subprocess.run(
        ["./target/release/rustsheet-eval", "show", CASE, "--all"], check=True, capture_output=True, text=True
    ).stdout
    res: dict[str, dict[str, str]] = {}
    key = None
    for line in out.splitlines():
        m = CELL.match(line)
        if m:
            key = m[1]
            res[key] = {}
            continue
        m = VAL.match(line)
        if m and key:
            res[key][m[1]] = m[2]
    return res


def from_report(path: str) -> dict[str, dict[str, str]]:
    oracle = json.loads(
        Path("eval/suites", CASE.replace("/", "/cases/").replace(".case.json", ".oracle.json")).read_text()
    )["sheets"]["Calc"]
    res = {f"Calc!{k}": {"actual": repr(v["v"])} for k, v in oracle.items()}
    (case,) = (c for c in json.loads(Path(path).read_text())["cases"] if c["id"] == CASE)
    for cell in case["cells"]:
        res[f"{cell['sheet']}!{cell['cell']}"] = {"actual": repr(cell["actual"]["v"])}
    return res


def within15(excel: float, ours: float) -> bool:
    if excel == ours:
        return True
    if not (math.isfinite(excel) and math.isfinite(ours)) or excel == 0:
        return False
    return abs(ours - excel) <= 0.5 * 10 ** (math.floor(math.log10(abs(excel))) - 14)


def in_envelope(math_v: mpmath.mpf, excel: float, ours: float) -> bool:
    if within15(excel, ours):
        return True
    err = abs(mpmath.mpf(ours) - math_v)
    return bool(err <= max(EPS * abs(math_v), FLOOR))


def _bits(v: float) -> int:
    return int(struct.unpack("<q", struct.pack("<d", v))[0])


def main() -> None:
    mpmath.mp.dps = 50
    if "--save" in sys.argv:
        label = sys.argv[sys.argv.index("--save") + 1]
        cells = (
            show() if "--report" not in sys.argv else from_report(sys.argv[sys.argv.index("--report") + 1])
        )
        (HERE / f"sweep-{label}.json").write_text(
            json.dumps({k: v["actual"] for k, v in sorted(cells.items())}, indent=1) + "\n"
        )
        return
    case_path = Path("eval/suites", CASE.replace("/", "/cases/"))
    oracle = json.loads(case_path.with_name(case_path.name.replace(".case.json", ".oracle.json")).read_text())
    formulas = json.loads(case_path.read_text())["sheets"][0]["cells"]
    cells = {
        f"Calc!{k}": {"formula": f, "expected": repr(oracle["sheets"]["Calc"][k]["v"])}
        for k, f in formulas.items()
    }
    ref = {k: mpmath.mpf(v) for k, v in json.loads((HERE / "mpmath.json").read_text()).items()}
    excel = {k: float(v["expected"]) for k, v in cells.items()}
    x_of = {k: float(re.search(r"\(([^,]+),", v["formula"])[1]) / 2 for k, v in cells.items()}
    runs = {"correctly rounded": {k: float(ref[k]) for k in cells}}
    for path in sorted(HERE.glob("sweep-*.json")):
        runs[path.stem.removeprefix("sweep-")] = {
            k: float(v) for k, v in json.loads(path.read_text()).items()
        }
    excel_cr = sum(excel[k] == float(ref[k]) for k in cells)
    print(f"{len(cells)} cells; Excel is the correctly rounded double at {excel_cr}")
    for part, keep in (
        ("all", lambda _x: True),
        ("x <= 30", lambda x: x <= 30),
        ("x > 30", lambda x: x > 30),
    ):
        keys = [k for k in cells if keep(x_of[k])]
        print(f"  {part} ({len(keys)} cells): label bit-exact within15 envelope max-ulp-vs-math")
        for label, vals in runs.items():
            exact = sum(vals[k] == excel[k] for k in keys)
            w15 = sum(within15(excel[k], vals[k]) for k in keys)
            env = sum(in_envelope(ref[k], excel[k], vals[k]) for k in keys)
            worst = max((abs(_bits(vals[k]) - _bits(float(ref[k]))) for k in keys), default=0)
            print(f"    {label:>18} {exact:5d} {w15:5d} {env:5d} {worst:6d}")


if __name__ == "__main__":
    main()
