"""T-3269 decision 0188 R4.1 hold-out and R4.2 fresh sweep, reproducible: the split, the values, the scoring.

Cells: every literal Q(a, x) cell with an oracle value in eval/suites (`=POISSON(k,x,TRUE)`,
`=POISSON.DIST(k,x,TRUE)`, a = floor(k) + 1; `=CHISQ.DIST.RT(x,df)`, `=CHIDIST(x,df)`, a = floor(df) / 2
and x / 2), with 1 < a < 20 and x >= a + 1, except the post-freeze fresh sweep t3269-fresh-sweep-2 (R4.2's
measure; the first sweep, t3269-fresh-sweep, was filled before this rework, so it is hold-out evidence now).
With `--only CASE` the cells are that case's alone, split "sweep" (the R4.2 score).

Split: test when int(sha256(f"{a!r},{x!r}").hexdigest()[:8], 16) % 100 < 30, by (a, x) so a POISSON cell
and its CHISQ twin fall together; the fit set (probes-numeric/t3269-gamma-tail-switch, the 145 cells that
placed FAR_TAIL_MIN_X) is train whatever its hash, so no fitted cell is in test.

  cells [--only CASE]    write P-split.tsv (case, cell, formula, a, x, hash, split, excel)
  mpmath                 write P-mpmath.json: Q(a, x) per (a, x) at 50 digits (mpmath.gammainc)
  save LABEL BIN         write P-LABEL.json: BIN's value per cell (`BIN show --all`, a copied binary)
  report [LABEL ...]     per split: exact, within 4 ulp, within 15 digits (decision 0083), Gate A envelope
                         (decision 0188 R2: 1e-9 relative of the math or within 15 digits of Excel) and the
                         largest relative error against the math, for each label and the correctly rounded Q;
                         `--by-shape` adds the far range's bit-exact count per shape
  near ACCURATE FORM     (FORM = `unrestricted`, the form at every shape, 952d0357b) per shape, cells next to the cut (30 < x <= 45) where FORM is closer to Excel than
                         ACCURATE, and farther, by more than the oracle's 4-ulp floor: FAR_TAIL_MAX_SHAPE's evidence
  --prefix P             file prefix (default holdout; the R4.2 sweep uses fresh2)
  --dir PATH             where the files live (default: beside this script)

Usage (repo root): tools/.venv/bin/python -I docs/research/t3269/holdout.py [--prefix P] cells|mpmath|save|report|near
"""

from __future__ import annotations

import hashlib
import json
import math
import re
import struct
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

import mpmath

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
SUITES = ROOT / "eval/suites"
FIT_CASE = "probes-numeric/t3269-gamma-tail-switch.case.json"
EXCLUDE = ("t3269-fresh-sweep-2",)
NUM = r"([0-9]*\.?[0-9]+(?:[eE][+-]?[0-9]+)?)"
POISSON = re.compile(rf"^=POISSON(?:\.DIST)?\({NUM},{NUM},(?:TRUE|1)\)$")
CHISQ = re.compile(rf"^=(?:CHISQ\.DIST\.RT|CHIDIST)\({NUM},{NUM}\)$")
SHOW_CELL = re.compile(r"^(\S+!\S+)\s")
SHOW_VAL = re.compile(r"^\s+actual\s*:\s*(\S+)")
REL_MIN = 1e-290  # below it the envelope's absolute floor, not a relative error, applies
EPS, FLOOR, FAR_X, MAX_X, MAX_SHAPE, NEAR_X, ULP_FLOOR = 1e-9, 1e-300, 30.0, 600.0, 11.0, 45.0, 4


def shape_x(formula: str) -> tuple[float, float] | None:
    if m := POISSON.match(formula):
        return math.floor(float(m[1])) + 1.0, float(m[2])
    if m := CHISQ.match(formula):
        return math.floor(float(m[2])) / 2.0, float(m[1]) / 2.0
    return None


def bucket(a: float, x: float) -> int:
    return int(hashlib.sha256(f"{a!r},{x!r}".encode()).hexdigest()[:8], 16) % 100


def discover(only: str | None) -> list[dict[str, str]]:
    rows = []
    for path in sorted(SUITES.glob("*/cases/*.case.json")):
        cid = f"{path.parent.parent.name}/{path.name}"
        if (only is None and path.name.removesuffix(".case.json") in EXCLUDE) or only not in (None, cid):
            continue
        oracle_path = path.with_name(path.name.replace(".case.json", ".oracle.json"))
        if not oracle_path.exists():
            continue
        case = json.loads(path.read_text())
        oracle = json.loads(oracle_path.read_text()).get("sheets", {})
        for sheet in case.get("sheets", []):
            for cell, formula in sheet.get("cells", {}).items():
                if not isinstance(formula, str) or (ax := shape_x(formula.replace(" ", ""))) is None:
                    continue
                a, x = ax
                v = oracle.get(sheet["name"], {}).get(cell, {})
                if v.get("t") != "n" or not (1.0 < a < 20.0 and x >= a + 1.0):
                    continue
                h = bucket(a, x)
                split = "sweep" if only else "train" if cid == FIT_CASE or h >= 30 else "test"
                rows.append(
                    {
                        "case": cid,
                        "cell": f"{sheet['name']}!{cell}",
                        "formula": formula,
                        "a": repr(a),
                        "x": repr(x),
                        "hash": str(h),
                        "split": split,
                        "excel": repr(float(v["v"])),
                    }
                )
    return rows


def read_split(d: Path, prefix: str = "holdout") -> list[dict[str, str]]:
    lines = (d / f"{prefix}-split.tsv").read_text().splitlines()
    head = lines[0].split("\t")
    return [dict(zip(head, line.split("\t"))) for line in lines[1:]]


def show(binary: str, case: str) -> dict[str, str]:
    out = subprocess.run(
        [binary, "--root", str(ROOT), "show", case, "--all"], check=True, capture_output=True, text=True
    ).stdout
    res, key = {}, None
    for line in out.splitlines():
        if m := SHOW_CELL.match(line):
            key = m[1]
        elif (m := SHOW_VAL.match(line)) and key:
            res[key] = m[1]
    return res


def bits(v: float) -> int:
    i = int(struct.unpack("<q", struct.pack("<d", v))[0])
    return i if i >= 0 else -(i & 0x7FFFFFFFFFFFFFFF)


def within15(excel: float, ours: float) -> bool:
    if excel == ours:
        return True
    if not (math.isfinite(excel) and math.isfinite(ours)) or excel == 0:
        return False
    return abs(ours - excel) <= 0.5 * 10 ** (math.floor(math.log10(abs(excel))) - 14)


def envelope(ref: mpmath.mpf, excel: float, ours: float) -> bool:
    return within15(excel, ours) or bool(abs(mpmath.mpf(ours) - ref) <= max(EPS * abs(ref), FLOOR))


def ulps(a: float, b: float) -> int:
    return abs(bits(a) - bits(b))


def score(rows, vals, refs) -> tuple[int, int, int, int, int, float]:
    n = len(rows)
    ex = sum(vals[r["key"]] == float(r["excel"]) for r in rows)
    u4 = sum(ulps(vals[r["key"]], float(r["excel"])) <= ULP_FLOOR for r in rows)
    w15 = sum(within15(float(r["excel"]), vals[r["key"]]) for r in rows)
    env = sum(envelope(refs[r["ax"]], float(r["excel"]), vals[r["key"]]) for r in rows)
    rel = max(
        (float(abs(mpmath.mpf(vals[r["key"]]) / refs[r["ax"]] - 1)) for r in rows if refs[r["ax"]] > REL_MIN),
        default=0.0,
    )
    return n, ex, u4, w15, env, rel


def form_range(r: dict[str, str]) -> bool:
    return FAR_X < float(r["x"]) <= MAX_X and float(r["a"]) <= MAX_SHAPE


def main() -> None:
    args = sys.argv[1:]
    opts = {}
    for flag in ("--dir", "--prefix", "--only"):
        if flag in args:
            i = args.index(flag)
            opts[flag] = args[i + 1]
            del args[i : i + 2]
    d, prefix = Path(opts.get("--dir", HERE)), opts.get("--prefix", "holdout")
    by_shape = "--by-shape" in args
    args = [a for a in args if a != "--by-shape"]
    mpmath.mp.dps = 50
    cmd = args[0]
    if cmd == "cells":
        rows = discover(opts.get("--only"))
        cols = ["case", "cell", "formula", "a", "x", "hash", "split", "excel"]
        text = "\t".join(cols) + "\n" + "".join("\t".join(r[c] for c in cols) + "\n" for r in rows)
        (d / f"{prefix}-split.tsv").write_text(text)
        print(len(rows), "cells;", sum(r["split"] == "test" for r in rows), "test")
        return
    rows = read_split(d, prefix)
    for r in rows:
        r["key"] = f"{r['case']}|{r['cell']}"
        r["ax"] = f"{r['a']},{r['x']}"
    if cmd == "mpmath":
        refs = {}
        for r in rows:
            if r["ax"] not in refs:
                q = mpmath.gammainc(mpmath.mpf(r["a"]), mpmath.mpf(r["x"]), mpmath.inf, regularized=True)
                refs[r["ax"]] = mpmath.nstr(q, 30)
        (d / f"{prefix}-mpmath.json").write_text(json.dumps(refs, indent=0, sort_keys=True) + "\n")
        return
    if cmd == "save":
        label, binary = args[1], args[2]
        keys = {r["key"] for r in rows}
        out = {}
        for case in sorted({r["case"] for r in rows}):
            out.update({f"{case}|{c}": v for c, v in show(binary, case).items() if f"{case}|{c}" in keys})
        (d / f"{prefix}-{label}.json").write_text(json.dumps(out, indent=0, sort_keys=True) + "\n")
        return

    def load(label: str) -> dict[str, float]:
        return {k: float(v) for k, v in json.loads((d / f"{prefix}-{label}.json").read_text()).items()}

    if cmd == "near":
        accurate, form = load(args[1]), load(args[2])
        print(f"30 < x <= {NEAR_X:g}, gaps over {ULP_FLOOR} ulp: shape | split closer/farther ...")
        tally: dict[tuple[float, str], list[int]] = defaultdict(lambda: [0, 0])
        for r in rows:
            if FAR_X < float(r["x"]) <= NEAR_X:
                e = float(r["excel"])
                uf, ua = ulps(form[r["key"]], e), ulps(accurate[r["key"]], e)
                t = tally[(float(r["a"]), r["split"])]
                t[0] += uf + ULP_FLOOR < ua
                t[1] += uf > ua + ULP_FLOOR
        splits = sorted({s for _, s in tally})
        for a in sorted({a for a, _ in tally}):
            print(f"{a:5.1f} | " + "  ".join(f"{s} {tally[(a, s)][0]}/{tally[(a, s)][1]}" for s in splits))
        for s in splits:
            for lo, hi in ((1.5, MAX_SHAPE), (MAX_SHAPE + 0.5, 19.5)):
                cl = sum(v[0] for (a, sp), v in tally.items() if sp == s and lo <= a <= hi)
                fa = sum(v[1] for (a, sp), v in tally.items() if sp == s and lo <= a <= hi)
                n = sum(sp == s and lo <= float(r["a"]) <= hi and FAR_X < float(r["x"]) <= NEAR_X for r in rows
                        for sp in [r["split"]])
                print(f"{s} shapes {lo:g}..{hi:g}: {n} cells, closer {cl}, farther {fa}")
        return
    refs = {k: mpmath.mpf(v) for k, v in json.loads((d / f"{prefix}-mpmath.json").read_text()).items()}
    runs = {"correctly rounded": {r["key"]: float(refs[r["ax"]]) for r in rows}}
    runs.update({label: load(label) for label in args[1:]})
    splits = [s for s in ("train", "test", "sweep") if any(r["split"] == s for r in rows)]
    print(
        f"{len(rows)} cells; fit set {sum(r['case'] == FIT_CASE for r in rows)}, "
        f"of which in test: {sum(r['case'] == FIT_CASE and r['split'] == 'test' for r in rows)}"
    )
    parts = [
        ("all", lambda r: True),
        ("x <= 30", lambda r: float(r["x"]) <= FAR_X),
        ("form's range (30 < x <= 600, a <= 11)", form_range),
        ("x > 30 outside it", lambda r: float(r["x"]) > FAR_X and not form_range(r)),
    ]
    head = " | ".join(f"{s}: n exact <=4ulp 15dig envelope max-rel-err" for s in splits)
    for part, keep in parts:
        print(f"{part}: label | {head}")
        for label, vals in runs.items():
            cols = []
            for split in splits:
                n, ex, u4, w15, env, rel = score([r for r in rows if keep(r) and r["split"] == split], vals, refs)
                cols.append(f"{n:4d} {ex:4d} {u4:4d} {w15:4d} {env:4d} {rel:9.2e}")
            print(f"  {label:>18} | " + " | ".join(cols))
    if by_shape:
        print("x > 30, bit-exact per shape (all splits)")
        groups = defaultdict(list)
        for r in rows:
            if float(r["x"]) > FAR_X:
                groups[float(r["a"])].append(r)
        print("   a    n " + " ".join(f"{lab[:10]:>10}" for lab in runs))
        for a in sorted(groups):
            sel = groups[a]
            print(f"{a:5.1f} {len(sel):4d} " + " ".join(f"{score(sel, v, refs)[1]:10d}" for v in runs.values()))


if __name__ == "__main__":
    main()
