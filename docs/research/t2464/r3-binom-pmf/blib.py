"""T-2513 r3-binom-pmf: shared helpers for Excel's BINOM.DIST(k,n,p,FALSE) and NEGBINOM.DIST(f,r,p,FALSE).

Data: the pmf cells of ../cells.tsv (BINOM.DIST / NEGBINOM.DIST with cumulative False and plain numeric args) plus
this directory's probes (probe_<name>.meta.json + the eval oracle sidecar).  Exact values from Fractions.

Model pieces reuse r3-loader-bd0's r3lib (numpy x87 arithmetic, Excel LN memo, EXP V6, Loader bd0 with knobs) and
r2-log1p's Excel log1p by import.  Every model returns E (the exponent EXP is applied to) or the value itself.
"""

from __future__ import annotations

import csv, hashlib, json, math, pickle, sys  # noqa: E401
from fractions import Fraction
from pathlib import Path

import numpy as np

HERE = Path(__file__).parent
T = HERE.parent
for p in (T / "r3-loader-bd0", T / "r2-log1p", T):
    if str(p) not in sys.path:
        sys.path.insert(0, str(p))

import mpmath  # noqa: E402
import r3lib as R3  # noqa: E402
import excel_log1p as XL1P  # noqa: E402

LDB = np.longdouble
st = R3.st
LN = R3.LN
CASES = R3.CASES
mpmath.mp.prec = 200


# ---------------------------------------------------------------- data
def _num(s: str):
    try:
        return float(s)
    except ValueError:
        return None


def load_cells(fns=("BINOM.DIST", "BINOMDIST", "NEGBINOM.DIST", "NEGBINOMDIST")):
    """pmf rows of cells.tsv: dict(fn, case, cell, k, n, p, excel).  NEGBINOM rows carry f, r in k, n."""
    rows = []
    with open(T / "cells.tsv") as f:
        for r in csv.DictReader(f, delimiter="\t"):
            if r["fn"] not in fns:
                continue
            a = r["args"].split("|")
            neg = r["fn"].startswith("NEG")
            if neg and r["fn"] == "NEGBINOMDIST":
                if len(a) != 3:
                    continue
                cum = "False"
            else:
                if len(a) != 4:
                    continue
                cum = a[3]
            if cum != "False":
                continue
            k, n, p = (_num(v) for v in a[:3])
            ex = _num(r["excel"])
            if None in (k, n, p, ex):
                continue
            if k != int(k) or n != int(n) or not (0 < p < 1) or not (0 <= k):
                continue
            if not neg and k > n:
                continue
            rows.append(
                dict(
                    fn="NEG" if neg else "BIN",
                    case=r["case"],
                    cell=r["cell"],
                    k=int(k),
                    n=int(n),
                    p=p,
                    excel=ex,
                )
            )
    return rows


def _probe(name: str):
    import pmfprobe as mkprobe

    cid, _, meta = mkprobe.build(name)
    side = CASES / f"{cid}.oracle.json"
    sh = json.loads(side.read_text())["sheets"]["Calc"] if side.exists() else {}
    return cid, meta, sh


def load_probe(name: str):
    """Rows of one of this directory's probes (row map rebuilt by mkprobe.build): fn BIN, BETA or NEG; k, n, p."""
    cid, meta, sh = _probe(name)
    rows = []
    for cell, fn, k, n, p, tag in meta["rows"]:
        c = sh.get(cell)
        if c and c.get("t") == "n":
            rows.append(dict(fn=fn, case=cid, cell=cell, k=k, n=n, p=p, excel=float(c["v"]), blk=tag))
    return rows


def echo_check(name: str):
    """Echo cells whose value differs from the intended double (parser misreads)."""
    cid, meta, sh = _probe(name)
    return [
        (cell, want, sh.get(cell))
        for cell, want in meta["echo"]
        if not sh.get(cell) or sh[cell].get("t") != "n" or float(sh[cell]["v"]) != want
    ]


def md5odd(r) -> bool:
    return int(hashlib.md5(f"{r['k']}|{r['n']}|{r['p']!r}".encode()).hexdigest(), 16) & 1 == 1


# ---------------------------------------------------------------- exact values
_EX: dict = {}


def exact_pmf(fn: str, k: int, n: int, p: float) -> mpmath.mpf:
    """Exact binomial pmf C(n,k) p^k (1-p)^(n-k) (BIN) or NEGBINOM pmf C(f+r-1,f) p^r (1-p)^f (k=f, n=r)."""
    key = (fn, k, n, p)
    v = _EX.get(key)
    if v is None:
        P = Fraction(p)
        top, low, a, b = (n, k, k, n - k) if fn == "BIN" else (k + n - 1, k, n, k)
        if top <= 3000:
            lc = mpmath.log(math.comb(top, low))
        else:  # math.comb is too slow for the large-n probe rows
            lc = mpmath.loggamma(top + 1) - mpmath.loggamma(low + 1) - mpmath.loggamma(top - low + 1)
        lv = (
            lc
            + a * mpmath.log(mpmath.mpf(P.numerator) / P.denominator)
            + b * mpmath.log(mpmath.mpf((1 - P).numerator) / (1 - P).denominator)
        )
        v = _EX[key] = mpmath.exp(lv)
    return v


# ---------------------------------------------------------------- scoring
def ulp_err(m: float, e: float) -> float:
    if m == e:
        return 0.0
    return (m - e) / math.ulp(e)


def within15(m: float, e: float) -> bool:
    if m == e:
        return True
    if not (math.isfinite(m) and math.isfinite(e)) or e == 0:
        return False
    half = 0.5 * 10.0 ** (math.floor(math.log10(abs(e))) - 14)
    return abs(m - e) <= half


def tally(pairs):
    """(exact, within 4 ulp, within 15 digits, n) for (model, excel) pairs."""
    ex = w4 = w15 = n = 0
    for m, e in pairs:
        n += 1
        if m is None or not math.isfinite(m):
            continue
        ex += m == e
        w4 += abs(ulp_err(m, e)) <= 4
        w15 += within15(m, e)
    return ex, w4, w15, n


def fmt(t) -> str:
    return f"{t[0]}/{t[1]}/{t[2]} of {t[3]}"


# ---------------------------------------------------------------- Excel pieces
def log1p_xl(a: float) -> float:
    """Excel's ln(1+a): the 14-term series for -0.375 <= a < 0, else LN(fl(1+a))."""
    if -0.375 <= a < 0:
        return XL1P.log1p_series(a)
    return LN(st(LDB(1) + LDB(a)))


SF = [  # R's sferr_halves: stirlerr(n/2), n = 0..30
    0.0,
    0.1534264097200273452913848,
    0.0810614667953272582196702,
    0.0548141210519176538961390,
    0.0413406959554092940938221,
    0.03316287351993628748511048,
    0.02767792568499833914878929,
    0.02374616365629749597132920,
    0.02079067210376509311152277,
    0.01848845053267318523077934,
    0.01664469118982119216319487,
    0.01513497322191737887351255,
    0.01387612882307074799874573,
    0.01281046524292022692424986,
    0.01189670994589177009505572,
    0.01110455975820691732662991,
    0.010411265261972096497478567,
    0.009799416126158803298389475,
    0.009255462182712732917728637,
    0.008768700134139385462952823,
    0.008330563433362871256469318,
    0.007934114564314020547248100,
    0.007573675487951840794972024,
    0.007244554301320383179543912,
    0.006942840107209529865664152,
    0.006665247032707682442354394,
    0.006408994188004207068439631,
    0.006171712263039457647532867,
    0.005951370112758847735624416,
    0.005746216513010115682023589,
    0.005554733551962801371038690,
]
S = [1 / 12, 1 / 360, 1 / 1260, 1 / 1680, 1 / 1188]
LN_2PI = float(mpmath.log(2 * mpmath.pi))
LN_SQRT_2PI = float(mpmath.log(mpmath.sqrt(2 * mpmath.pi)))

_STE: dict = {}


def stirlerr_exact(n) -> mpmath.mpf:
    n = float(n)
    v = _STE.get(n)
    if v is None:
        N = mpmath.mpf(n)
        v = _STE[n] = (
            mpmath.loggamma(N + 1) - (N + 0.5) * mpmath.log(N) + N - mpmath.log(mpmath.sqrt(2 * mpmath.pi))
        )
    return v


def stirlerr(n: float, mode: str):
    """stirlerr(n) as an extended value.  mode: 'cr' exact rounded to double, 'ex' exact (to 64 bits),
    'R' R's table (n <= 15) + stored series with cut-offs 15/35/80/500, 'Re' same series extended,
    'Rcr' table + correctly rounded above 15."""
    if mode == "ex":
        return R3._ld_of(stirlerr_exact(n))
    if mode == "cr":
        return LDB(float(stirlerr_exact(n)))
    if n <= 15:
        nn = 2 * n
        if nn == int(nn):
            return LDB(SF[int(nn)])
        return LDB(float(stirlerr_exact(n)))
    if mode == "Rcr":
        return LDB(float(stirlerr_exact(n)))
    x = (lambda v: LDB(st(v))) if mode == "R" else (lambda v: v)
    N = LDB(n)
    nn = x(N * N)
    s0, s1, s2, s3, s4 = (LDB(v) for v in S)
    if n > 500:
        return x(x(s0 - x(s1 / nn)) / N)
    if n > 80:
        return x(x(s0 - x(x(s1 - x(s2 / nn)) / nn)) / N)
    if n > 35:
        return x(x(s0 - x(x(s1 - x(x(s2 - x(s3 / nn)) / nn)) / nn)) / N)
    return x(x(s0 - x(x(s1 - x(x(s2 - x(x(s3 - x(s4 / nn)) / nn)) / nn)) / nn)) / N)


# ---------------------------------------------------------------- the Loader pmf with knobs
BD0_DEFAULT = dict(cut=0.2, ld="x", v="x", ph=1)  # r3-loader-bd0's model


def loader_E(k: int, n: int, p: float, q: float, o: dict):
    """E of dbinom_raw(k, n, p, q) for 0 < k < n.  Knobs in o (besides bd0's, passed to r3lib.bd0):
    stir: stirlerr mode; cs: 'x' Cs stored / 'e' extended; lc: 'x' stored lc = fl(Cs - b1 - b2), 'e' extended;
    lf: 'LF0' ln2pi + LN k + LN(n-k) - LN n, 'LF1' ln2pi + LN k + log1p(-k/n), 'ex' exact;
    chain: 'x' E = fl(lc - lf/2), 'e' extended."""
    bo = dict(BD0_DEFAULT)
    bo.update(o.get("bd0", {}))
    K, N, NK = LDB(k), LDB(n), LDB(n - k)
    npx = R3.R(N * LDB(p), o.get("np", "x"))
    nqx = (
        R3.R(N * LDB(q), o.get("np", "x")) if o.get("nq", "mul") == "mul" else R3.R(N - npx, o.get("np", "x"))
    )
    b1, _ = R3.bd0(K, npx, bo)
    b2, _ = R3.bd0(NK, nqx, bo)
    sm = o.get("stir", "ex")
    s_n, s_k, s_nk = stirlerr(n, sm), stirlerr(k, sm), stirlerr(n - k, sm)
    cso = o.get("cs", "e")
    order = o.get("order", "R")
    if order == "R":  # stirlerr(n) - stirlerr(k) - stirlerr(n-k) - bd0 - bd0, left to right
        v = R3.R(s_n - s_k, cso)
        v = R3.R(v - s_nk, cso)
        v = R3.R(v - b1, o.get("lc", "e"))
        lc = R3.R(v - b2, o.get("lc", "e"))
    else:  # Cs then X
        Cs = R3.R(R3.R(s_n - s_k, cso) - s_nk, cso)
        X = R3.R(-b1 - b2, o.get("xs", "e"))
        lc = R3.R(Cs + X, o.get("lc", "e"))
    lfm = o.get("lf", "LF0")
    if lfm == "LF0":
        lf = st(LDB(st(LDB(st(LDB(LN_2PI) + LDB(LN(float(k))))) + LDB(LN(float(n - k))))) - LDB(LN(float(n))))
    elif lfm == "LF1":
        lf = st(LDB(st(LDB(LN_2PI) + LDB(LN(float(k))))) + LDB(log1p_xl(-st(K / N))))
    else:
        lf = float(mpmath.log(2 * mpmath.pi * k * (n - k) / mpmath.mpf(n)))
    h = LDB(lf) / 2
    E = lc - h
    return E if o.get("chain", "x") == "e" else LDB(st(E))


def loader_pmf(k: int, n: int, p: float, o: dict, q: float | None = None) -> float:
    if q is None:
        q = st(LDB(1) - LDB(p))
    if k == 0:
        return R3.exp_v6(LDB(st(LDB(n) * LDB(log1p_xl(-p)))))
    if k == n:
        return R3.exp_v6(LDB(st(LDB(n) * LDB(LN(p)))))
    return R3.exp_v6(loader_E(k, n, p, q, o))


# ---------------------------------------------------------------- Excel GAMMALN (engine model), cached here
_GLP = HERE / "gl.pkl"
_GL: dict = pickle.loads(_GLP.read_bytes()) if _GLP.exists() else {}


def GL(vs):
    """Excel's GAMMALN of each double in vs (batched, cached)."""
    import excelmath

    need = sorted({float(v) for v in vs if float(v) not in _GL})
    if need:
        for v, r in zip(need, excelmath.excel("GAMMALN", need)):
            _GL[v] = r
        _GLP.write_bytes(pickle.dumps(_GL))
    return [_GL[float(v)] for v in vs]


def D_units(r) -> float:
    """(ln Excel - ln exact) * 2^53: the exponent error Excel's value carries (output rounding adds <= 1 unit)."""
    ex = exact_pmf(r["fn"], r["k"], r["n"], r["p"])
    return float((mpmath.log(r["excel"]) - mpmath.log(ex)) * 2**53)


def units(E, r) -> float:
    """(model E - exact ln pmf) * 2^53 for an extended E."""
    ex = exact_pmf(r["fn"], r["k"], r["n"], r["p"])
    return float(
        (
            mpmath.mpf(Fraction(*LDB(E).as_integer_ratio()).numerator)
            / Fraction(*LDB(E).as_integer_ratio()).denominator
            - mpmath.log(ex)
        )
        * 2**53
    )


# ---------------------------------------------------------------- free-constant fits (r3-loader-bd0's interval method)
def ab_of(r):
    """Beta shapes of a binomial row: a = k+1, b = n-k+1 (Loader's k = a-1, n = a+b-2)."""
    return float(r["k"] + 1), float(r["n"] - r["k"] + 1)


def exact_ln(r) -> mpmath.mpf:
    if r["fn"] == "BETA":
        return mpmath.log(exact_pmf("BIN", r["k"], r["n"], r["p"])) + mpmath.log(r["n"] + 1)
    if r["fn"] == "NEG":  # NEGBINOM.DIST(n-k, k, p): C(n-1, n-k) p^k q^(n-k) = (k/n) * binomial pmf
        return mpmath.log(exact_pmf("BIN", r["k"], r["n"], r["p"])) + mpmath.log(mpmath.mpf(r["k"]) / r["n"])
    return mpmath.log(exact_pmf(r["fn"], r["k"], r["n"], r["p"]))


def fit_block(rows, idx, o: dict, chain: str = "C", W: float = 1024.0):
    """One free constant C per block (E = chain(C, X)); fit on the even md5 half, score the odd half.
    X = -bd0(k, np) - bd0(n-k, nq) from r3lib.xpart with knobs o."""
    r0 = rows[idx[0]]
    a, b = ab_of(r0)
    Xs = {
        i: (
            xpart_v(rows[i]["p"], rows[i]["k"], rows[i]["n"], o)
            if "mr" in o
            else R3.xpart(rows[i]["p"], a, b, o)
        )
        for i in idx
    }
    i0 = idx[len(idx) // 2]
    b1, b2, _ = Xs[i0]
    L5, h = (0.0, 0.0) if chain.rstrip("e") in ("C", "SC") else shape_Lh(r0, o)
    C0 = LDB(float(exact_ln(rows[i0]))) + b1 + b2 - LDB(L5) + LDB(h)
    ivs = {i: R3.cell_intervals(rows[i], Xs[i][0], Xs[i][1], chain, C0, L5, h, W) for i in idx}
    return _block_summary(rows, idx, ivs, Xs, C0)


def shape_Lh(r, o):
    """(L5, h) of a row for the r3lib chains: L5 = LN(n+1) for BETA rows (0 otherwise), h = lf/2, lf by o['lf']."""
    k, n = r["k"], r["n"]
    L5 = LN(float(n + 1)) if r["fn"] == "BETA" else 0.0
    lfm = o.get("lf", "LF0")
    if lfm == "LF0":
        lf = st(LDB(st(LDB(st(LDB(LN_2PI) + LDB(LN(float(k))))) + LDB(LN(float(n - k))))) - LDB(LN(float(n))))
    elif lfm == "LF1":
        lf = st(LDB(st(LDB(LN_2PI) + LDB(LN(float(k))))) + LDB(log1p_xl(-st(LDB(k) / LDB(n)))))
    else:
        lf = float(mpmath.log(2 * mpmath.pi * k * (n - k) / mpmath.mpf(n)))
    return L5, lf / 2


def blocks(rows, fn=None):
    out: dict = {}
    for i, r in enumerate(rows):
        if fn is None or r["fn"] == fn:
            out.setdefault((r["fn"], r["blk"]), []).append(i)
    return out


# ---------------------------------------------------------------- x part with M-formation variants
def bd0_v(k, n, p, side: str, o: dict):
    """bd0(k', M) for side 'p' (k' = k, M ~ n p) or 'q' (k' = n-k, M ~ n q).  o['mr'] picks how the log form
    forms its ratio and linear term:  'xx' r = fl(k'/fl(nP)), lin = fl(nP) - k'  (r3 model);
    'ee' r = fl(k'/nP_ext), lin = nP_ext - k';  'ex' r from extended, lin from stored;  'xe' the reverse;
    'kn' r = fl(fl(k'/n)/P);  'kn2' r = fl(fl(k'/P)/n).  P is p or q = fl(1-p).  Series sides use r3lib.bd0."""
    bo = dict(BD0_DEFAULT)
    bo.update(o.get("bd0", {}))
    K = LDB(k if side == "p" else n - k)
    P = LDB(p) if side == "p" else LDB(st(LDB(1) - LDB(p)))
    N = LDB(n)
    Me = N * P
    Mx = LDB(st(Me))
    if side == "q" and o.get("nq", "mul") == "sub":  # n q formed as n - fl(n p)
        Me = N - LDB(st(N * LDB(p)))
        Mx = LDB(st(Me))
    elif side == "q" and o.get("nq") == "subx":  # exact n - n p (extended)
        Me = N - N * LDB(p)
        Mx = LDB(st(Me))
    d = LDB(st(K - Mx))
    sm = LDB(st(K + Mx))
    if abs(d) < LDB(bo.get("cut", 0.2)) * sm:
        return R3.bd0(K, Mx, bo)[0], "s"
    mr = o.get("mr", "xx")
    if mr in ("xx", "xe"):
        r = st(K / Mx)
    elif mr in ("ee", "ex"):
        r = st(K / Me)
    elif mr == "kn":
        r = st(LDB(st(K / N)) / P)
    else:
        r = st(LDB(st(K / P)) / N)
    if o.get("rq", "x") == "x":  # stored ratio, Excel LN stored
        lr = LDB(LN(r))
    elif o["rq"] == "l":  # stored ratio, LN kept extended (FYL2X unstored)
        lr = R3.LN64(r)
    else:  # ratio and LN extended
        lr = R3.LN64(K / (Mx if mr in ("xx", "xe") else Me))
    t = R3.R(K * lr, o.get("tm", "x"))
    lin = (Mx - K) if mr in ("xx", "ex", "kn", "kn2") else (Me - K)
    if o.get("lin", "x") == "x":
        lin = LDB(st(lin))
    return R3.R(t + lin, o.get("bs", "e")), "l"


def xpart_v(p, k, n, o):
    b1, s1 = bd0_v(k, n, p, "p", o)
    b2, s2 = bd0_v(k, n, p, "q", o)
    return b1, b2, s1 + s2


# ---------------------------------------------------------------- general chains: E from C, b1, b2, h with storage flags
def _s(v, f):
    return LDB(st(v)) if f == "x" else v


def chain_g(spec: str, C, b1, b2, h):
    """spec = order:flags.  Orders (C is the free constant, here Cs = stirlerr part):
    'R'  ((C - b1) - b2) - h        (R's lc, then lc - lf/2)       flags s1 s2 s3
    'X'  (C - (b1 + b2)) - h        (x part summed first)          flags s1 s2 s3 (s1 = b1+b2)
    'H'  (C - h) - (b1 + b2)        (shape constant first)         flags s1 s2 s3 (s1 = C-h, s2 = b1+b2)
    'B'  b1, b2 stored first, then R order                          flags s1 s2 s3
    """
    order, fl = spec.split(":", 1)
    if order == "T":  # a summation tree by its tree_name, e.g. T:(C+(H+(P+Q)e)x)e
        return tree_eval(_tree_by_name(fl), dict(C=C, H=-LDB(h), P=-b1, Q=-b2))
    s1, s2, s3 = fl
    if order == "R":
        return _s(_s(_s(C - b1, s1) - b2, s2) - LDB(h), s3)
    if order == "B":
        b1, b2 = LDB(st(b1)), LDB(st(b2))
        return _s(_s(_s(C - b1, s1) - b2, s2) - LDB(h), s3)
    if order == "X":
        return _s(_s(C - _s(b1 + b2, s1), s2) - LDB(h), s3)
    if order == "H":
        return _s(_s(C - LDB(h), s1) - _s(b1 + b2, s2), s3)
    raise ValueError(spec)


def fit_block_g(rows, idx, o: dict, spec: str, W: float = 1024.0):
    """fit_block for chain_g specs (one free C per block, even-half fit, odd-half score)."""
    r0 = rows[idx[0]]
    Xs = {i: xpart_v(rows[i]["p"], rows[i]["k"], rows[i]["n"], o) for i in idx}
    L5, h = shape_Lh(r0, o)
    i0 = idx[len(idx) // 2]
    b1, b2, _ = Xs[i0]
    C0 = LDB(float(exact_ln(rows[i0]))) + b1 + b2 - LDB(L5) + LDB(h)
    fin = o.get("fin", "e")  # BETA rows: E = L5 + chain (extended 'e' or stored 'x')

    def ivl(i):
        b1, b2, _ = Xs[i]

        def F(u):
            E = chain_g(spec, C0 + LDB(u) * R3.U53, b1, b2, h)
            if L5:
                E = _s(LDB(L5) + E, fin)
            return R3.exp_v6(E)

        y = rows[i]["excel"]
        iv = R3.off_interval(F, y, y, W)
        return [iv] if iv and F(iv[0]) == y else []

    ivs = {i: ivl(i) for i in idx}
    return _block_summary(rows, idx, ivs, Xs, C0)


# ---------------------------------------------------------------- chains as summation trees over (C, -h, -b1, -b2)
def tree_specs():
    """Every binary summation tree over the four terms C, H (= -h), P (= -b1), Q (= -b2) with a store flag per
    internal node ('x' stored, 'e' extended), as nested tuples (flag, left, right); leaves are letters."""
    import itertools

    def trees(items):
        if len(items) == 1:
            yield items[0]
            return
        items = list(items)
        first = items[0]
        rest = items[1:]
        for r in range(0, len(rest)):
            for comb in itertools.combinations(rest, r):
                left = (first,) + comb
                right = tuple(x for x in rest if x not in comb)
                for lt in trees(left):
                    for rt in trees(right):
                        yield (lt, rt)

    def flags(t):
        if isinstance(t, str):
            yield t
            return
        for a in flags(t[0]):
            for b in flags(t[1]):
                for f in "xe":
                    yield (f, a, b)

    seen = set()
    for t in trees(("C", "H", "P", "Q")):
        for ft in flags(t):
            if ft not in seen:
                seen.add(ft)
                yield ft


_TREES: dict = {}


def _tree_by_name(nm: str):
    if not _TREES:
        _TREES.update({tree_name(t): t for t in tree_specs()})
    return _TREES[nm]


def tree_eval(t, env):
    if isinstance(t, str):
        return env[t]
    f, a, b = t
    v = tree_eval(a, env) + tree_eval(b, env)
    return LDB(st(v)) if f == "x" else v


def tree_name(t) -> str:
    if isinstance(t, str):
        return t
    f, a, b = t
    return f"({tree_name(a)}+{tree_name(b)}){f}"


# ---------------------------------------------------------------- R 2.0 / Loader (2000) non-log form: exp(lc) / sqrt(f)
M_2PI = float(2 * mpmath.pi)


def fx_g(k: int, n: int, o: dict):
    """g = sqrt(f), f = (2 pi k (n-k)) / n (R_D_fexp).  o['f']: 'R' fl(fl(fl(2pi*k)*(n-k))/n) stored ops,
    'Re' same extended, 'ex' exact; o['sq']: 'x' stored sqrt, 'e' extended (FSQRT of f)."""
    fm = o.get("f", "R")
    K, NK, N = LDB(k), LDB(n - k), LDB(n)
    if fm == "ex":
        f = R3._ld_of(mpmath.mpf(2) * mpmath.pi * k * (n - k) / n)
    else:
        X = (lambda v: LDB(st(v))) if fm == "R" else (lambda v: v)
        f = X(X(X(LDB(M_2PI) * K) * NK) / N)
    g = np.sqrt(f)
    return LDB(st(g)) if o.get("sq", "x") == "x" else g


def fx_F(k: int, n: int, o: dict):
    """Loader (2000)'s factor F = sqrt(n / (PI2*x*(n-x))).  o['F']: a string of 4 flags for the stores of
    PI2*x, *(n-x), n/., sqrt ('x' stored, 'e' extended), default 'eeee'."""
    fl = o.get("F", "eeee")
    K, NK, N = LDB(k), LDB(n - k), LDB(n)
    a = _s(LDB(M_2PI) * K, fl[0])
    b = _s(a * NK, fl[1])
    c = _s(N / b, fl[2])
    return _s(np.sqrt(c), fl[3])


def fx_out(lc, g, o: dict) -> float:
    """o['fin'] 'div': out = fl(EXP(lc) / g);  'mul': out = fl(EXP(lc) * F) (Loader 2000), F = fx_F.
    EXP's result extended (o['ex'] 'e') or stored ('x')."""
    e = LDB(R3.exp_v6(lc, o.get("ex", "e") == "e"))
    if o.get("fin", "div") == "mul":
        return st(e * g)
    return st(e / g)


def lc_chain(spec: str, C, b1, b2):
    """lc from C (stirlerr part), b1, b2: 'R:ab' = ((C - b1)a - b2)b; 'X:ab' = (C - (b1 + b2)a)b."""
    order, fl = spec.split(":")
    if order == "R":
        return _s(_s(C - b1, fl[0]) - b2, fl[1])
    return _s(C - _s(b1 + b2, fl[0]), fl[1])


def fit_block_fx(rows, idx, o: dict, spec: str = "R:ex", W: float = 1024.0):
    """Free C per block for out = fl(EXP(lc)/g) (BETA rows: times (n+1) per o['beta'])."""
    r0 = rows[idx[0]]
    k, n = r0["k"], r0["n"]
    Xs = {i: xpart_v(rows[i]["p"], k, n, o) for i in idx}
    mul = o.get("fin", "div") == "mul"
    g = fx_F(k, n, o) if mul else fx_g(k, n, o)
    i0 = idx[len(idx) // 2]
    b1, b2, _ = Xs[i0]
    lnex = exact_pmf("BIN", k, n, rows[i0]["p"])
    lg = float(mpmath.log(mpmath.mpf(float(g))))
    C0 = LDB(float(mpmath.log(lnex))) + b1 + b2 + LDB(-lg if mul else lg)

    def ivl(i):
        b1, b2, _ = Xs[i]
        r = rows[i]

        def F(u):
            lc = lc_chain(spec, C0 + LDB(u) * R3.U53, b1, b2)
            v = fx_out(lc, g, o)
            if r["fn"] == "BETA":
                v = beta_from(v, lc, g, n, o)
            elif r["fn"] == "NEG":
                v = neg_from(v, lc, g, k, n, o)
            return v

        y = r["excel"]
        iv = R3.off_interval(F, y, y, W)
        return [iv] if iv and F(iv[0]) == y else []

    ivs = {i: ivl(i) for i in idx}
    return _block_summary(rows, idx, ivs, Xs, C0)


def beta_from(v, lc, g, n, o):
    """BETA.DIST density from the binomial pieces: o['beta'] 'mul' fl((n+1) * pmf), 'mule' fl((n+1)*EXP(lc)/g)
    with one final store."""
    m = o.get("beta", "mul")
    if m == "mul":
        return st(LDB(n + 1) * LDB(v))
    e = R3.exp_v6(lc, True)
    return st(LDB(n + 1) * e / g)


def neg_from(v, lc, g, k, n, o):
    """NEGBINOM pmf from the binomial pieces: o['neg'] 'mul' fl(fl(k/n) * pmf), 'div' fl(k * pmf / n) ..."""
    m = o.get("neg", "mul")
    if m == "mul":
        return st(LDB(st(LDB(k) / LDB(n))) * LDB(v))
    if m == "kn":
        return st(LDB(st(LDB(k) * LDB(v))) / LDB(n))
    return st(LDB(k) * LDB(v) / LDB(n))


def h_variants(k: int, n: int) -> dict:
    """Candidate h = lf/2 doubles (Excel LN, x87 stored ops)."""
    K, NK, N = LDB(k), LDB(n - k), LDB(n)
    x = lambda v: LDB(st(v))  # noqa: E731
    out = {}
    out["LF0"] = shape_Lh(dict(fn="BIN", k=k, n=n), dict(lf="LF0"))[1]
    out["LF1"] = shape_Lh(dict(fn="BIN", k=k, n=n), dict(lf="LF1"))[1]
    f_R = x(x(x(LDB(M_2PI) * K) * NK) / N)  # R 2.0: f = (M_2PI*x*(n-x))/n
    out["lnf_R"] = LN(float(f_R)) / 2
    f_P = x(N / x(x(LDB(M_2PI) * K) * NK))  # Loader: n/(PI2*x*(n-x)); h = -0.5 ln of it
    out["lnf_P"] = -LN(float(f_P)) / 2
    f_2 = x(x(K * NK) / N)  # ln(2pi) + LN(k (n-k)/n)
    out["ln2pi+LN(knk/n)"] = st(LDB(LN_2PI) + LDB(LN(float(f_2)))) / 2
    f_3 = x(LDB(M_2PI) * x(x(K * NK) / N))
    out["LN(2pi*(knk/n))"] = LN(float(f_3)) / 2
    out["LF0 reordered (lnk+lnnk)-lnn+ln2pi"] = (
        st(LDB(st(LDB(st(LDB(LN(float(k))) + LDB(LN(float(n - k))))) - LDB(LN(float(n))))) + LDB(LN_2PI)) / 2
    )
    out["LN_SQRT_2PI + 0.5(..)"] = st(
        LDB(LN_SQRT_2PI) + LDB(st(LDB(st(LDB(LN(float(k))) + LDB(LN(float(n - k))))) - LDB(LN(float(n))))) / 2
    )
    return out


# ---------------------------------------------------------------- frozen full models (no free constants)
MODELS = {  # name: (tree over C, H=-h, P=-b1, Q=-b2; lf form; stirlerr mode)
    "L-ext/LF0": ("(((C+P)e+Q)e+H)e", "LF0", "R"),
    "L-ext/LF1": ("(((C+P)e+Q)e+H)e", "LF1", "R"),
    "L-Sx/LF0": ("(C+(H+(P+Q)e)x)e", "LF0", "R"),
    "L-lcx/LF0": ("(((C+P)e+Q)x+H)e", "LF0", "R"),
    "L-lcx/LF1": ("(((C+P)e+Q)x+H)e", "LF1", "R"),
    "L-lcx/LF0/ex": ("(((C+P)e+Q)x+H)e", "LF0", "ex"),
    "L-lcxx/LF0": ("(((C+P)x+Q)x+H)e", "LF0", "R"),
    "L-lcx/lnfR": ("(((C+P)e+Q)x+H)e", "lnf_R", "R"),
}


def model_value(name: str, fn: str, k: int, n: int, p: float) -> float:
    """Excel's BINOM/BETA/NEGBINOM pmf under a frozen model.  k = 0 and k = n use the closed forms
    EXP(fl(n log1p(-p))) and EXP(fl(n LN p)); BETA rows add LN(n+1) to E (extended); NEG rows are
    fl(p * BINOM(k-1, n-1, p)) (the shifted kernel)."""
    if fn == "NEG":
        return st(LDB(p) * LDB(model_value(name, "BIN", k - 1, n - 1, p)))
    if name == "CR":
        v = exact_pmf("BIN", k, n, p)
        return float(v * (n + 1)) if fn == "BETA" else float(v)
    if fn == "BIN" and k == 0:
        return R3.exp_v6(LDB(st(LDB(n) * LDB(log1p_xl(-p)))))
    if fn == "BIN" and k == n:
        return R3.exp_v6(LDB(st(LDB(n) * LDB(LN(p)))))
    tree, lf, sm = MODELS[name]
    b1, b2, _ = xpart_v(p, k, n, dict(mr="xx"))
    C = _s(stirlerr(n, sm) - stirlerr(k, sm), "e") - stirlerr(n - k, sm)
    h = h_variants(k, n)[lf] if lf.startswith("lnf") else shape_Lh(dict(fn="BIN", k=k, n=n), dict(lf=lf))[1]
    E = tree_eval(_tree_by_name(tree), dict(C=C, H=-LDB(h), P=-b1, Q=-b2))
    if fn == "BETA":
        E = LDB(LN(float(n + 1))) + E
    return R3.exp_v6(E)


def _block_summary(rows, idx, ivs, Xs, C0):
    """One block's interval fit: the best common offset on the even (md5) half, its hits on the odd half, and the
    all-cell overlap, with branch counts (shared by the shape-constant scans above)."""
    ev = [i for i in idx if not md5odd(rows[i])]
    od = [i for i in idx if md5odd(rows[i])]
    ne, ce = R3.best_overlap([ivs[i] for i in ev])
    nh = R3.hits_at([ivs[i] for i in od], ce)
    na, ca = R3.best_overlap([ivs[i] for i in idx])
    br: dict = {}
    for i in idx:
        br[Xs[i][2]] = br.get(Xs[i][2], 0) + 1
    return dict(even=ne, nev=len(ev), hold=nh, nod=len(od), all=na, n=len(idx), off=ce, off_all=ca, br=br, C0=C0,
                ivs=ivs, X=Xs)
