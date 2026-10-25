"""verify-r5-f-combine: re-score the frozen F-density candidates with this directory's parser and tally.
Worker functions (imported, checked against vmodel's re-implementation first):
  M-sdm  r4-f-small-df sdmodel.value(x, d1, d2, 'mulr');  M-sd  sdmodel.value(x, d1, d2, 'div')
  hkp-sd, hyb  r5-f-combine final_score.value(row, model)
Usage: vscore.py CASE_ID... | --full (every probes-numeric F.DIST pdf case) [--check]
Prints per model: all, r > 0.375, r <= 0.375 and max(d) classes; with --check, worker vs re-implementation."""

from __future__ import annotations

import math
import sys

import mpmath

import vmodel as VM
import vparse as VP
import vtally as VT

sys.path.insert(0, str(VM.T / "r5-f-combine"))
import final_score as FS  # noqa: E402
import sdmodel as SD  # noqa: E402

MODELS = ("CR", "M-sdm", "M-sd", "hkp-sd", "hyb")


def cr(r) -> float:
    """Correctly rounded exact density (mpmath, 60 digits)."""
    with mpmath.workdps(60):
        x, m, n = (mpmath.mpf(r[k]) for k in ("x", "d1", "d2"))
        a, b = m / 2, n / 2
        lf = a * mpmath.log(m * x / n) - (a + b) * mpmath.log1p(m * x / n) - mpmath.log(x) - mpmath.log(mpmath.beta(a, b))
        return float(mpmath.exp(lf))


def worker(r, mdl: str) -> float:
    if mdl == "CR":
        return cr(r)
    if mdl == "M-sdm":
        return float(SD.value(r["x"], r["d1"], r["d2"], "mulr"))
    if mdl == "M-sd":
        return float(SD.value(r["x"], r["d1"], r["d2"], "div"))
    return float(FS.value(r, mdl))


def classes(r) -> tuple:
    _, rr = VM.side_r(r["x"], r["d1"], r["d2"])
    md = max(r["d1"], r["d2"])
    return ("all", "r>0.375" if rr > 0.375 else "r<=0.375", "d<=12" if md <= 12 else "d<=60" if md <= 60 else "d>60")


def score(rows, check: bool):
    VM.prefetch_ln([v for r in rows for v in VM.ln_inputs(r["x"], r["d1"], r["d2"])])
    for mdl in MODELS:
        g: dict = {}
        agree = [0, 0]
        for r in rows:
            v = worker(r, mdl)
            for k in classes(r):
                g.setdefault(k, []).append((v, r["excel"]))
            if check and mdl not in ("hyb", "CR") and classes(r)[1] == "r>0.375":
                agree[0] += 1
                agree[1] += VM.value(r["x"], r["d1"], r["d2"], mdl) == v
        line = " | ".join(f"{k} {VT.fmt(VT.tally(p))}" for k, p in sorted(g.items()))
        print(f"  {mdl:7s} {line}" + (f" | reimpl agrees {agree[1]}/{agree[0]}" if agree[0] else ""))
        sys.stdout.flush()


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    check = "--check" in sys.argv
    if "--full" in sys.argv:
        args = VP.fdist_cases()
    allrows = []
    for c in args:
        rows = [r for r in VP.rows(c) if math.isfinite(r["excel"])]
        if not rows:
            continue
        print(f"{c}: {len(rows)} cells")
        score(rows, check)
        allrows += rows
    if len(args) > 1:
        print(f"TOTAL: {len(allrows)} cells")
        score(allrows, False)


if __name__ == "__main__":
    main()
