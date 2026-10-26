"""The closed-form rule and its variants on all closed-form cells, split by the README branch (direct: the kernel's
x <= a/(a+b); reflected otherwise) and hold-out half.  Variant knobs:
  orient : 'oriented' (swap at x > 0.5, as bratio) | 'unoriented' (a == 1: 1-(1-x)^b ; b == 1: x^a on the caller's x)
  ln1mx  : a0 = 1 log argument: 'LN(1-x0)' | 'LN(y0)'
  compl  : a0 = 1 complement: '(0.5-w1)+0.5' | '1-w1'
  cut    : a0 = 1 applies when 'y0<=0.9' | 'x0>=0.1' | 'always'"""
import itertools, math, sys
from collections import defaultdict
sys.path.insert(0, '.')
import w5lib as L
from closed_final import items

def val(a, b, x, y, side, orient, ln1mx, compl, cut):
    swap = (x > 0.5) if orient == "oriented" else False
    a0, b0, x0, y0 = (b, a, y, x) if swap else (a, b, x, y)
    if b0 == 1:
        w = L.EXP(a0 * L.LN(x0)); w1 = (0.5 - w) + 0.5
    elif a0 == 1 and (cut == "always" or (cut == "y0<=0.9" and y0 <= 0.9) or (cut == "x0>=0.1" and x0 >= 0.1)):
        w1 = L.EXP(b0 * L.LN(1 - x0 if ln1mx == "LN(1-x0)" else y0))
        w = (0.5 - w1) + 0.5 if compl == "(0.5-w1)+0.5" else 1 - w1
    else:
        return None
    lower, upper = (w1, w) if swap else (w, w1)
    return lower if side == "lower" else upper

cells = list(items())
# the cell set: every cell where the reference rule applies (oriented, y0<=0.9)
ref = [(it, val(it[4], it[5], it[2], it[3], it[6], "oriented", "LN(1-x0)", "(0.5-w1)+0.5", "y0<=0.9")) for it in cells]
ref = [(it, m) for it, m in ref if m is not None]
print("cells where the reference rule applies:", len(ref))
variants = list(itertools.product(["oriented", "unoriented"], ["LN(1-x0)", "LN(y0)"], ["(0.5-w1)+0.5", "1-w1"], ["y0<=0.9", "x0>=0.1", "always"]))
print(f"{'orient':10s} {'ln(1-x0)':9s} {'compl':13s} {'cut':8s} | {'direct':18s} {'reflected':18s} {'hold-out':18s} {'all':18s}")
for v in variants:
    grp = defaultdict(list)
    for (src, c, x, y, a, b, side, ex), _ in ref:
        m = val(a, b, x, y, side, *v)
        br = "direct" if x <= a / (a + b) else "reflected"
        grp[br].append((m, ex)); grp["all"].append((m, ex))
        if L.holdout(c["case"], c["cell"]): grp["hold"].append((m, ex))
    print(f"{v[0]:10s} {v[1]:9s} {v[2]:13s} {v[3]:8s} | " + " ".join(f"{L.fmt(L.score(grp[k])):18s}" for k in ("direct", "reflected", "hold", "all")))
