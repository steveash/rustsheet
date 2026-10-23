"""Build ix_evidence.tsv: every oracle cell of F.DIST(TRUE)/F.DIST.RT/FDIST, T.DIST(TRUE)/T.DIST.RT/T.DIST.2T/TDIST,
BINOM.DIST(TRUE)/BINOMDIST(TRUE), NEGBINOM.DIST(TRUE), plus the reference-argument BETA.DIST cells of
t0969-t-beta-link, mapped onto Excel's incomplete-beta kernel K(a, b, x, y) (x and y = the two arguments the caller
forms; y is the complement the kernel receives).  See RESULTS.md section 2 for the evidence behind every mapping.

Columns: source_fn case cell x a b side excel_ix confidence | y excel composition branch computed_tail closed exact
err_ulp ix_lo ix_hi
  side          lower: excel_ix is Excel's I_x(a,b); upper: excel_ix is Excel's 1 - I_x(a,b)
  excel_ix      the implied tail value (exact derivation from the cell, e.g. 2L for T); for 'interval' rows the
                midpoint, with [ix_lo, ix_hi] every tail value the cell is consistent with
  confidence    direct      : this tail is the kernel's directly computed output (bratio dispatch, section 2.2)
                derived     : this tail is (0.5 - w) + 0.5 of the computed one (exact relation, but rounded)
                closed      : a kernel closed form (b0 = 1, or a0 = 1 with y0 <= 0.9), verified to the bit (closed_final.py)
                interval    : the tail comes through a lossy wrapper op (T centre form, 1 - L)
                uncertain   : mapping not verified for this region (see composition)
                not-kernel  : the cell does not go through the kernel (BINOM k = 0)
  branch        TOMS 708 bratio branch of K(a, b, x, y) (dispatch.py); 'closed:*' for closed forms
  computed_tail which tail of I_x(a,b) the kernel computes directly
  exact         exact tail (mpmath, 40 digits), taking the oriented small argument (x if x <= 0.5, else y) as given\n                and the other as its exact complement; empty for max(a,b) > 2000
  err_ulp       (excel_ix - exact)/ulp(excel_ix)
"""
import json, math, sys
import mpmath
sys.path.insert(0, '.')
import w5lib as L
from dispatch import bratio_branch

OUT = "ix_evidence.tsv"


def closed_kind(a, b, x, y):
    swap = x > 0.5
    a0, b0, x0, y0 = (b, a, y, x) if swap else (a, b, x, y)
    if b0 == 1:
        return "closed:b0=1", ("lower" if not swap else "upper")
    if a0 == 1 and y0 <= 0.9:
        return "closed:a0=1", ("upper" if not swap else "lower")
    return None, None


def classify(a, b, x, y, side):
    ck, comp = closed_kind(a, b, x, y)
    if ck:
        return ck, comp, "closed"
    br, ind, comp = bratio_branch(a, b, x, y)
    return br, comp, ("direct" if comp == side else "derived")


_n_exact = [0]


_TC = {}
_TCF = "tail_cache.pkl"
try:
    import pickle
    _TC = pickle.load(open(_TCF, "rb"))
except Exception:
    _TC = {}


def exact_tail(x, y, a, b, side):
    """The requested tail without cancellation: lower = I_{x*}(a,b), upper = I_{y*}(b,a), where the oriented small
    argument is taken as given (x if x <= 0.5, else y) and the other one is its exact complement."""
    if max(a, b) > 2000:
        return None
    k = (x, y, a, b, side)
    if k in _TC:
        return _TC[k]
    _n_exact[0] += 1
    if _n_exact[0] % 1000 == 0:
        import pickle
        pickle.dump(_TC, open(_TCF, "wb"))
    X = mpmath.mpf(x) if x <= 0.5 else 1 - mpmath.mpf(y)
    Y = 1 - mpmath.mpf(x) if x <= 0.5 else mpmath.mpf(y)
    with mpmath.workdps(40):
        v = mpmath.betainc(a, b, 0, X, regularized=True) if side == "lower" else mpmath.betainc(b, a, 0, Y, regularized=True)
    _TC[k] = v
    return v


rows = []


def emit(fn, c, x, y, a, b, side, ix, conf, comp_txt, lo=None, hi=None, excel=None, override=None):
    if not (0 < x < 1) or not (0 < y < 1) or not math.isfinite(ix):
        return
    br, comp, cf = classify(a, b, x, y, side)
    if override:
        cf = override
    elif conf:
        cf = conf if cf != "closed" else cf
    ex = exact_tail(x, y, a, b, side)
    if ex is None:
        exs, err = "", float("nan")
    else:
        exs = mpmath.nstr(ex, 20)
        err = (ix - float(ex)) / math.ulp(ix) if ix != 0 else float("nan")
    rows.append([fn, c["case"] if isinstance(c, dict) else c[0], c["cell"] if isinstance(c, dict) else c[1],
                 repr(x), repr(a), repr(b), side, repr(ix), cf, repr(y), repr(excel if excel is not None else c["excel"]),
                 comp_txt, br, comp, "1" if br.startswith("closed") else "0", exs, f"{err:.2f}",
                 "" if lo is None else repr(lo), "" if hi is None else repr(hi)])


# ---------------------------------------------------------------- F
for c in L.cells({"F.DIST", "F.DIST.RT", "FDIST"}):
    a_ = c["args"]
    if c["fn"] == "F.DIST" and (len(a_) < 4 or a_[3] is not True):
        continue
    X, d1, d2 = a_[0], float(math.trunc(a_[1])), float(math.trunc(a_[2]))
    if not (X > 0 and 1 <= d1 < 1e10 and 1 <= d2 < 1e10):
        continue
    p = d1 * X
    w = 1 / (1 + d2 / p); v = 1 / (1 + p / d2)
    side = "lower" if c["fn"] == "F.DIST" else "upper"
    ex = c["excel"]
    conf = "saturated" if ex in (0.0, 1.0) else None
    emit(c["fn"], c, w, v, d1 / 2, d2 / 2, side, ex, conf,
         "K(d1/2,d2/2,x=1/(1+d2/(d1*X)),y=1/(1+(d1*X)/d2)); other tail = (0.5-w)+0.5", override=conf)

# ---------------------------------------------------------------- T
def t_rows():
    for c in L.cells({"T.DIST", "T.DIST.RT", "T.DIST.2T", "TDIST"}):
        a_ = c["args"]
        fn = c["fn"]; v = c["excel"]
        t, n = a_[0], math.trunc(a_[1])
        if n < 1 or n >= 1e10 or t == 0:
            continue
        if fn == "T.DIST":
            if len(a_) < 3 or a_[2] is not True:
                continue
            if t < 0: Lt, how = v, "L=T.DIST(-|t|)"
            else: Lt, how = None, "L=1-T.DIST(t)"
        elif fn == "T.DIST.RT":
            if t > 0: Lt, how = v, "L=T.DIST.RT"
            else: Lt, how = None, "L=1-T.DIST.RT(-|t|)"
        elif fn == "T.DIST.2T":
            if t < 0: continue
            Lt, how = v / 2, "L=T.DIST.2T/2"
        else:
            tails = math.trunc(a_[2])
            if tails not in (1, 2) or t < 0: continue
            Lt, how = v / tails, f"L=TDIST/{tails}"
        yield c, fn, abs(t), float(n), Lt, how, v


for c, fn, t, n, Lt, how, v in t_rows():
    tt = t * t; s = n + tt; y = n / s; z = tt / s
    lossy = Lt is None
    if lossy:
        Lt = 1 - v   # exact in double only where v >= 0.5; the interval below covers the rounding
        uL = math.ulp(v)
        Llo, Lhi = 1 - v - uL / 2, 1 - v + uL / 2
    else:
        Llo = Lhi = Lt
    if Lt <= 0:
        continue
    if n == 2 and 1 - y <= 0.9:
        emit(fn, c, y, 1 - y, 1.0, 0.5, "lower", 2 * Lt, None,
             f"df2 closed form: I_y(1,1/2) = 2L, y=fl(2/fl(2+t*t)), complement fl(1-y); {how}",
             lo=2 * Llo if lossy else None, hi=2 * Lhi if lossy else None, excel=v,
             override="interval" if lossy else None)
    elif n == 2:
        emit(fn, c, y, 1 - y, 1.0, 0.5, "lower", 2 * Lt, None,
             f"df2, y<0.1: general kernel at y (complement unverified); {how}",
             lo=2 * Llo if lossy else None, hi=2 * Lhi if lossy else None, excel=v,
             override="interval" if lossy else "uncertain")
    elif t > 1 and y <= 0.99:
        br, ind, comp = bratio_branch(n / 2, 0.5, y, 1 - y)
        over = "interval" if lossy else ("uncertain" if br not in ("bpser", "bpser(a,b>1)") and comp == "lower" and False else None)
        emit(fn, c, y, 1 - y, n / 2, 0.5, "lower", 2 * Lt, None,
             f"t>1,y<=0.99: I_y(n/2,1/2) = 2L, y=fl(n/fl(n+fl(t*t))), complement fl(1-y) (bit-identical to BETA.DIST(y) in "
             f"bpser branches; 1-4 ulp apart in bup/bgrat branches); {how}",
             lo=2 * Llo if lossy else None, hi=2 * Lhi if lossy else None, excel=v, override=over)
    else:
        # L = fl(0.5 - H/2), H = I_z(1/2, n/2): H in [1 - 2L - u, 1 - 2L + u], u = ulp(L)
        u = math.ulp(Lhi)
        Hmid = 1 - 2 * Lt
        Hlo, Hhi = 1 - 2 * Lhi - u, 1 - 2 * Llo + u
        region = "t<=1" if t <= 1 else "t>1,y>0.99 (centre form only partly verified)"
        emit(fn, c, z, y, 0.5, n / 2, "lower", Hmid, None,
             f"{region}: L = 0.5 - I_z(1/2,n/2)/2, z=fl(fl(t*t)/fl(n+fl(t*t))), complement y; {how}",
             lo=Hlo, hi=Hhi, excel=v, override="interval" if t <= 1 else "uncertain")

# ---------------------------------------------------------------- BINOM / NEGBINOM
for c in L.cells({"BINOM.DIST", "BINOMDIST"}):
    a_ = c["args"]
    if len(a_) < 4 or a_[3] is not True:
        continue
    k, n, p = math.trunc(a_[0]), math.trunc(a_[1]), a_[2]
    if not (0 < p < 1 and 0 <= k < n):
        continue
    if k == 0:
        emit(c["fn"], c, p, 1 - p, 1.0, float(n), "upper", c["excel"], None,
             "k=0: = BINOM pmf(0) = EXP(fl(n*l)), l = Excel log1p(-p) (p<=0.375) or LN(1-p): not the kernel's closed form",
             override="not-kernel")
        continue
    emit(c["fn"], c, p, 1 - p, float(k + 1), float(n - k), "upper", c["excel"], None,
         "BINOM cdf = 1 - I_p(k+1,n-k) = I_q(n-k,k+1); kernel args x=p, y=fl(1-p) (orientation of a/b unverified)")

for c in L.cells({"NEGBINOM.DIST"}):
    a_ = c["args"]
    if len(a_) < 4 or a_[3] is not True:
        continue
    f, r, p = math.trunc(a_[0]), math.trunc(a_[1]), a_[2]
    if not (0 < p < 1 and f >= 0 and r >= 1):
        continue
    emit(c["fn"], c, p, 1 - p, float(r), float(f + 1), "lower", c["excel"], None,
         "NEGBINOM cdf = I_p(r,f+1); x=p, y=fl(1-p)")

# ---------------------------------------------------------------- BETA.DIST reference cells of t0969-t-beta-link
base = "/home/user/rustsheet/eval/suites/probes-numeric/cases/t0969-t-beta-link"
case = json.load(open(base + ".case.json")); orc = json.load(open(base + ".oracle.json"))
cells = case["sheets"][0]["cells"]; o = orc["sheets"]["Link"]
for r in range(1, 1000):
    if f"A{r}" not in cells:
        break
    t, n = float(cells[f"A{r}"]), float(cells[f"B{r}"])
    g = lambda k: o.get(f"{k}{r}", {}).get("v")
    D, G = g("D"), g("G")
    cid = ("probes-numeric/t0969-t-beta-link.case.json", None)
    for col, x, a, b in (("E", D, n / 2, 0.5), ("H", G, 0.5, n / 2), ("I", 1 - D, 0.5, n / 2)):
        val = g(col)
        if val is None or not (0 < x < 1):
            continue
        emit("BETA.DIST(ref)", (cid[0], f"Link!{col}{r}"), x, 1 - x, a, b, "lower", val, None,
             f"BETA.DIST({'D' if col=='E' else 'G' if col=='H' else '1-D'}{r},..,TRUE) with the worksheet's y/z; kernel y = fl(1-x)",
             excel=val)

import pickle
pickle.dump(_TC, open(_TCF, "wb"))
hdr = ["source_fn", "case", "cell", "x", "a", "b", "side", "excel_ix", "confidence", "y", "excel", "composition", "branch",
       "computed_tail", "closed", "exact", "err_ulp", "ix_lo", "ix_hi"]
with open(OUT, "w") as f:
    f.write("\t".join(hdr) + "\n")
    for r in rows:
        f.write("\t".join(r) + "\n")
from collections import Counter
cnt = Counter((r[0], r[8]) for r in rows)
for k in sorted(cnt): print(k, cnt[k])
print("rows", len(rows))
