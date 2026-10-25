"""Independent re-score of r2-bpser-small's top claim (GL53/glalgdiv, no fpser/apser) on the w_bpser cells.

Own code throughout: data parse (case formulas + oracle json, not the worker's meta), TOMS bratio dispatch for
min(a,b) <= 1, x87 stored arithmetic (numpy longdouble), Excel EXP V6 for an extended argument (validated against the
engine harness on doubles), Excel LN from the engine harness, Excel GAMMALN measured by the worker's -gl probe (parsed
here from its case + output; harness fallback counted), TOMS algdiv ported here.
usage: verify.py [old]     (old: also score beta_exact.tsv, fully independent of the worker's tuning)
"""

from __future__ import annotations

import hashlib
import json
import math
import pickle
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

import mpmath
import numpy as np

HERE = Path(__file__).resolve().parent
T = HERE.parent
sys.path.insert(0, str(T))
import excelmath as em  # noqa: E402  (shared tool: engine harness + fl64/fl53)

CASES = Path("/home/user/rustsheet/eval/suites/probes-numeric/cases")
W = T / "r2-bpser-small"
LD = np.longdouble
mpf = mpmath.mpf
mpmath.mp.prec = 200
EPS = 1e-15


def sx(op, a, b):
    p, q = LD(a), LD(b)
    return float(p + q if op == "+" else p - q if op == "-" else p * q if op == "*" else p / q)


# --------------------------------------------------------------- Excel elementary functions
LOG2E64 = em.fl64(1 / mpmath.ln(2))


def exp_ext(a) -> float:
    """Excel EXP V6 of an mpf argument (double or 64-bit extended)."""
    a = mpf(a)
    t = em.fl64(a * LOG2E64)
    n = mpmath.nint(t)
    f = t - n
    w = em.fl64(1 + em.fl64(mpf(2) ** abs(f) - 1))
    if f < 0:
        w = em.fl64(1 / w)
    return em.fl53(w * mpf(2) ** n)


_LN: dict = {}
_GL: dict = {}
GL_FALLBACK = set()


def need_ln(xs):
    xs = sorted({x for x in xs if x not in _LN})
    for i in range(0, len(xs), 20000):
        ch = xs[i : i + 20000]
        _LN.update(zip(ch, em.excel("LN", ch)))


def LN(x):
    return _LN[x]


def need_gl(vs):
    vs = sorted({v for v in vs if v not in _GL})
    GL_FALLBACK.update(vs)
    for i in range(0, len(vs), 20000):
        ch = vs[i : i + 20000]
        _GL.update(zip(ch, em.excel("GAMMALN", ch)))


def GL(v):
    return _GL[v]


# --------------------------------------------------------------- data
def parse_num(s: str) -> float:
    s = s.strip()
    m = re.fullmatch(r"\((\d+)\*2\^(-?\d+)\+(\d+)\*2\^(-?\d+)\)", s)
    if m:
        v = mpf(int(m[1])) * mpf(2) ** int(m[2]) + mpf(int(m[3])) * mpf(2) ** int(m[4])
        f = float(v)
        assert mpf(f) == v, s
        return f
    m = re.fullmatch(r"(\d+)\*2\^(-?\d+)", s)
    if m:
        return float(mpf(int(m[1])) * mpf(2) ** int(m[2]))
    return float(s)


def load_case(name):
    cells = json.load(open(CASES / f"{name}.case.json"))["sheets"][0]["cells"]
    out = json.load(open(W / f"{name}.out.json"))["sheets"]["Calc"]
    return cells, out


def load_probe(sets=("shapes", "runs", "tiny")):
    rows, bad = [], Counter()
    for nm in sets:
        cells, out = load_case(f"t2464-r2-bpser-small-{nm}")
        meta = {m[0]: m for m in json.load(open(W / f"t2464-r2-bpser-small-{nm}.meta.json"))}
        for addr, f in cells.items():
            m = re.fullmatch(r"=BETA\.DIST\((.+),([^,]+),([^,]+),TRUE\)", f)
            if not m:
                bad["formula"] += 1
                continue
            x, a, b = parse_num(m[1]), parse_num(m[2]), parse_num(m[3])
            i = int(addr[1:])
            mt = meta.get(i)
            if mt is None or (mt[2], mt[3], mt[4]) != (x, a, b):
                bad["meta-mismatch"] += 1
            c = out.get(addr)
            if not c or c.get("t") != "n":
                bad["non-numeric"] += 1
                continue
            rows.append(dict(set=nm, addr=addr, a=a, b=b, x=x, excel=float(c["v"])))
    return rows, bad


def load_gl_probe():
    cells, out = load_case("t2464-r2-bpser-small-gl")
    n = 0
    for addr, f in cells.items():
        m = re.fullmatch(r"=GAMMALN\((.+)\)", f)
        c = out.get(addr)
        if m and c and c.get("t") == "n":
            _GL[parse_num(m[1])] = float(c["v"])
            n += 1
    return n


def hold(a, b):
    """The worker's hold-out half (shape md5 odd), recomputed here."""
    return int(hashlib.md5(repr((a, b)).encode()).hexdigest(), 16) % 2 == 1


# --------------------------------------------------------------- dispatch (TOMS bratio, min(a,b) <= 1)
def dispatch(a, b, x):
    """('w_bpser', swap, a0, b0, x0) for the cells the claim covers, else (label, ...)."""
    y = 1.0 - x
    if min(a, b) > 1.0 or max(a, b) < EPS * 0.001:
        return ("other",)
    swap = x > 0.5
    a0, b0, x0, y0 = (b, a, y, x) if swap else (a, b, x, y)
    if b0 == 1.0 or (a0 == 1.0 and y0 <= 0.9):
        return ("closed",)
    if b0 < min(EPS, EPS * a0):
        return ("fpser",)
    if a0 < min(EPS, EPS * b0) and b0 * x0 <= 1.0:
        return ("apser",)
    if max(a0, b0) > 1.0:
        if b0 <= 1.0:
            return ("w_bpser", swap, a0, b0, x0)
        if x0 >= 0.3:
            return ("w1_bpser",)
        if x0 < 0.1 and math.pow(x0 * b0, a0) <= 0.7:
            return ("w_bpser", swap, a0, b0, x0)
        return ("bgrat",)
    if a0 >= min(0.2, b0) or math.pow(x0, a0) <= 0.9:
        return ("w_bpser", swap, a0, b0, x0)
    if x0 >= 0.3:
        return ("w1_bpser",)
    return ("bup",)


def sub_branch(a, b):
    lo, hi = min(a, b), max(a, b)
    if lo >= 1.0:
        return "a0>=1"
    return "b0<=1" if hi <= 1.0 else "1<b0<8" if hi < 8.0 else "b0>=8"


# --------------------------------------------------------------- model pieces
def series(a, b, x):
    """Forward series 1/a + sum c_n/(a+n), c_n = c_{n-1} * ((0.5 - b/n) + 0.5) * x, every op x87-stored; stops when
    the stored sum stops changing (at most 200,000 terms)."""
    total, coef = sx("/", 1.0, a), 1.0
    for k in range(1, 200001):
        ratio = sx("+", sx("-", 0.5, sx("/", b, float(k))), 0.5)
        coef = sx("*", sx("*", coef, ratio), x)
        nxt = sx("+", total, sx("/", coef, sx("+", a, float(k))))
        if nxt == total:
            return total
        total = nxt
    return total


def alnrel(a):
    if abs(a) > 0.375:
        return LN(1.0 + a)
    t = a / (a + 2.0)
    t2 = t * t
    w = (((-0.0178874546012214 * t2 + 0.405303492862024) * t2 - 1.29418923021993) * t2 + 1.0) / (
        ((-0.0845104217945565 * t2 + 0.747811014037616) * t2 - 1.62752256355323) * t2 + 1.0
    )
    return 2.0 * t * w


def algdiv(a, b):
    """TOMS 708 algdiv (double code), ln(Gamma(b)/Gamma(a+b)), b >= 8; LN = Excel's."""
    if a > b:
        h = b / a
        c = 1.0 / (1.0 + h)
        x = h / (1.0 + h)
        d = a + (b - 0.5)
    else:
        h = a / b
        c = h / (1.0 + h)
        x = 1.0 / (1.0 + h)
        d = b + (a - 0.5)
    x2 = x * x
    s3 = 1.0 + (x + x2)
    s5 = 1.0 + (x + x2 * s3)
    s7 = 1.0 + (x + x2 * s5)
    s9 = 1.0 + (x + x2 * s7)
    s11 = 1.0 + (x + x2 * s9)
    t = 1.0 / b
    t = t * t
    c0, c1, c2, c3, c4, c5 = (
        0.0833333333333333,
        -0.00277777777760991,
        7.9365066682539e-4,
        -5.9520293135187e-4,
        8.37308034031215e-4,
        -0.00165322962780713,
    )
    w = ((((c5 * s11 * t + c4 * s9) * t + c3 * s7) * t + c2 * s5) * t + c1 * s3) * t + c0
    w = w * (c / b)
    u = d * alnrel(a / b)
    v = a * (LN(b) - 1.0)
    if u > v:
        return (w - v) - u
    return (w - u) - v


def lnB(a0, b0):
    lo, hi = min(a0, b0), max(a0, b0)
    if hi < 8.0:
        return sx("+", GL(lo), sx("-", GL(hi), GL(sx("+", lo, hi))))
    return sx("+", GL(lo), algdiv(lo, hi))


def bpser(a, b, x):
    if x == 0.0:
        return 0.0
    s = series(a, b, x)
    alx = sx("*", a, LN(x))
    z = em.fl64(mpf(alx) - mpf(lnB(a, b)))
    return sx("*", exp_ext(z), s)


def model(a0, b0, x0, swap):
    w = bpser(a0, b0, x0)
    return 0.5 - w + 0.5 if swap else w


# --------------------------------------------------------------- scoring
def w15(m, e):
    if m == e:
        return True
    if not (math.isfinite(m) and e != 0):
        return False
    return abs(m - e) <= 0.5 * 10.0 ** (math.floor(math.log10(abs(e))) - 14)


def tally(pairs):
    n = ex = f4 = v = 0
    for m, e in pairs:
        n += 1
        ex += m == e
        f4 += m == e or (math.isfinite(m) and abs(m - e) <= 4 * math.ulp(e))
        v += w15(m, e)
    return n, ex, f4, v


def control_exp():
    rng = np.random.default_rng(7)
    xs = [float(v) for v in rng.uniform(-40, 0.5, 3000)] + [float(v) for v in rng.uniform(-1, 1, 1000)]
    got = em.excel("EXP", xs)
    return sum(exp_ext(mpf(v)) == g for v, g in zip(xs, got)), len(xs)


def prepare(rows):
    need_x, need_g = [], []
    for r in rows:
        d = dispatch(r["a"], r["b"], r["x"])
        r["d"] = d
        if d[0] == "w_bpser":
            _, sw, a0, b0, x0 = d
            lo, hi = min(a0, b0), max(a0, b0)
            need_x.append(x0)
            need_g += [lo, hi, sx("+", lo, hi)] if hi < 8 else [lo]
            if hi >= 8:
                need_x += [hi, 1.0 + lo / hi]
    need_ln(need_x)
    need_gl(need_g)


def report(rows, title, base=None):
    print(f"\n## {title}")
    groups = defaultdict(list)
    for i, r in enumerate(rows):
        if r["d"][0] == "w_bpser":
            groups[sub_branch(r["d"][2], r["d"][3])].append(i)
    for g in ("b0<=1", "1<b0<8", "b0>=8", "a0>=1"):
        idx = groups.get(g, [])
        if not idx:
            continue
        full = tally((rows[i]["m"], rows[i]["excel"]) for i in idx)
        ho = tally((rows[i]["m"], rows[i]["excel"]) for i in idx if rows[i]["hold"])
        tu = tally((rows[i]["m"], rows[i]["excel"]) for i in idx if not rows[i]["hold"])
        line = (
            f"{g:7s} n={full[0]} exact={full[1]} w4={full[2]} w15={full[3]} | hold n={ho[0]} ex={ho[1]} "
            f"w15={ho[3]} | tune n={tu[0]} ex={tu[1]}"
        )
        if base is not None:
            bf = sum(base[i] == rows[i]["excel"] for i in idx)
            bh = sum(base[i] == rows[i]["excel"] for i in idx if rows[i]["hold"])
            line += f" | baseline(worker W3z64) ex={bf} hold ex={bh}"
        sets = Counter(rows[i]["set"] for i in idx)
        print(line, dict(sets))


def main():
    print("EXP V6 control vs harness:", control_exp())
    ngl = load_gl_probe()
    print("GAMMALN probe values parsed:", ngl)
    rows, bad = load_probe()
    print("rows", len(rows), "problems", dict(bad))
    keys = Counter((r["a"], r["b"], r["x"]) for r in rows)
    print(
        "duplicate (a,b,x) keys:",
        sum(1 for k, v in keys.items() if v > 1),
        "extra cells",
        sum(v - 1 for v in keys.values()),
    )
    for r in rows:
        r["hold"] = hold(r["a"], r["b"])
    prepare(rows)
    print("GAMMALN args not in probe (harness model fallback):", len(GL_FALLBACK))
    fb_cells = 0
    for r in rows:
        d = r["d"]
        if d[0] == "w_bpser":
            r["m"] = model(d[2], d[3], d[4], d[1])
            lo, hi = min(d[2], d[3]), max(d[2], d[3])
            fb_cells += any(v in GL_FALLBACK for v in ([lo, hi, sx("+", lo, hi)] if hi < 8 else [lo]))
    print("w_bpser cells using a fallback GAMMALN:", fb_cells)
    # compare with the worker's pickle (their GL53/glalgdiv values and labels), cell by cell
    pk = pickle.load(open(W / "results_shapes_runs_tiny.pkl", "rb"))
    wkey = {(r["set"], r["a"], r["b"], r["x"]): i for i, r in enumerate(pk["rows"])}
    agree = disagree = lab_mis = 0
    base = []
    for r in rows:
        j = wkey.get((r["set"], r["a"], r["b"], r["x"]))
        base.append(pk["vals"]["W3z64"][j] if (j is not None and "W3z64" in pk["vals"]) else math.nan)
        if r["d"][0] != "w_bpser" or j is None:
            continue
        if not pk["labs"][j].startswith("w_bpser"):
            lab_mis += 1
        theirs = pk["vals"].get("GL53/glalgdiv", [None] * len(pk["rows"]))[j]
        if theirs is None:
            continue
        agree += theirs == r["m"]
        disagree += theirs != r["m"]
    print(f"vs worker pickle: same value {agree}, different {disagree}, label mismatch {lab_mis}")
    print("worker pickle variants:", list(pk["vals"])[:40])
    report(rows, "probe shapes+runs+tiny, w_bpser, GL53/glalgdiv (my implementation)", base)
    shp = [r for r in rows if r["set"] == "shapes"]
    report(shp, "shapes only")
    rn = [r for r in rows if r["set"] == "runs"]
    report(rn, "runs only")
    # per run
    print("\n## per run (exact / n)")
    per = defaultdict(lambda: [0, 0])
    for r in rn:
        if r["d"][0] == "w_bpser":
            k = (r["a"], r["b"])
            per[k][0] += r["m"] == r["excel"]
            per[k][1] += 1
    for k, v in per.items():
        print(k, sub_branch(k[0], k[1]), v, "hold" if hold(*k) else "tune")
    if "old" in sys.argv:
        old(rows)


def old(_):
    """beta_exact.tsv cdf cells: completely independent of the worker's probes and tuning."""
    import csv

    # GAMMALN measured by r2-bratio's probe at the old shapes (else harness model, counted)
    args = json.load(open(T / "r2-bratio" / "t2464-r2-bratio-gammaln.args.json"))
    out = json.load(open(T / "r2-bratio" / "t2464-r2-bratio-gammaln.out.json"))["sheets"]["Calc"]
    for i, v in enumerate(args):
        c = out.get(f"A{i + 1}")
        if c and c.get("t") == "n":
            _GL.setdefault(v, float(c["v"]))
    GL_FALLBACK.clear()
    rows = []
    for r in csv.DictReader(open(T / "beta_exact.tsv"), delimiter="\t"):
        if r["cum"] != "1":
            continue
        rows.append(
            dict(set="be", a=float(r["a"]), b=float(r["b"]), x=float(r["x"]), excel=float(r["excel"]))
        )
    for r in rows:
        r["hold"] = hold(r["a"], r["b"])
    prepare(rows)
    print("\nold: GAMMALN fallbacks", len(GL_FALLBACK))
    for r in rows:
        d = r["d"]
        if d[0] == "w_bpser":
            r["m"] = model(d[2], d[3], d[4], d[1])
    # baseline on the same cells: correctly rounded value
    cache = {}
    for r in rows:
        if r["d"][0] == "w_bpser":
            with mpmath.workdps(40):
                cache[id(r)] = float(
                    mpmath.betainc(mpf(r["a"]), mpf(r["b"]), 0, mpf(r["x"]), regularized=True)
                )
    report(rows, "OLD beta_exact.tsv cdf, w_bpser (fully out of sample)")
    groups = defaultdict(list)
    for r in rows:
        if r["d"][0] == "w_bpser":
            groups[sub_branch(r["d"][2], r["d"][3])].append(r)
    for g, rs in groups.items():
        print(g, "CR exact", sum(cache[id(r)] == r["excel"] for r in rs), "of", len(rs))


if __name__ == "__main__":
    main()
