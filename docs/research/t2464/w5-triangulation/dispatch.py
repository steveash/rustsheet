"""TOMS 708 bratio dispatch (R toms708.c), reduced to: which branch, whether the shapes are swapped, and which tail
is computed directly ('w' = the lower tail of the swapped problem, 'w1' its upper tail)."""
import math

EPS = 1e-15


def bratio_branch(a, b, x, y):
    """Return (branch, ind, computed) where computed in {'w', 'w1'} refers to the (possibly swapped) problem,
    and the original problem's directly computed tail is 'lower' or 'upper'."""
    eps = EPS
    ind = 0
    a0, b0, x0, y0 = a, b, x, y
    if min(a0, b0) <= 1:
        if x > 0.5:
            ind = 1; a0, b0, x0, y0 = b, a, y, x
        if b0 < min(eps, eps * a0):
            br, comp = "fpser", "w"
        elif a0 < min(eps, eps * b0) and b0 * x0 <= 1:
            br, comp = "apser", "w1"
        else:
            br = None
            if max(a0, b0) > 1:
                if b0 <= 1:
                    br, comp = "bpser", "w"
                elif x0 >= 0.29:
                    br, comp = "w1=bpser(b0,a0,y0)", "w1"
                elif x0 < 0.1 and (x0 * b0) ** a0 <= 0.7:
                    br, comp = "bpser", "w"
                elif b0 > 15:
                    br, comp = "bgrat", "w1"
            else:
                if a0 >= min(0.2, b0):
                    br, comp = "bpser", "w"
                elif x0 ** a0 <= 0.9:
                    br, comp = "bpser", "w"
                elif x0 >= 0.3:
                    br, comp = "w1=bpser(b0,a0,y0)", "w1"
            if br is None:
                br, comp = "bup+bgrat", "w1"
    else:
        lam = (a + b) * y - b if a > b else a - (a + b) * x
        if lam < 0:
            ind = 1; a0, b0, x0, y0 = b, a, y, x; lam = abs(lam)
        if b0 < 40:
            if b0 * x0 <= 0.7:
                br, comp = "bpser(a,b>1)", "w"
            else:
                br, comp = "L140 bup+bpser" if x0 <= 0.7 else "L140 bup+bgrat", "w"
        elif a0 > b0:
            br, comp = ("bfrac", "w") if (b0 <= 100 or lam > b0 * 0.03) else ("basym", "w")
        elif a0 <= 100:
            br, comp = "bfrac", "w"
        elif lam > a0 * 0.03:
            br, comp = "bfrac", "w"
        else:
            br, comp = "basym", "w"
    lower = (comp == "w") != bool(ind)
    return br, ind, "lower" if lower else "upper"


def load_link():
    """The t0969-t-beta-link probe: (case cells, oracle values of sheet Link)."""
    import json
    base = "/home/user/rustsheet/eval/suites/probes-numeric/cases/t0969-t-beta-link"
    case = json.load(open(base + ".case.json"))
    orc = json.load(open(base + ".oracle.json"))
    return case["sheets"][0]["cells"], orc["sheets"]["Link"]


def link_rows():
    """Each filled row r of the link probe as (t, n, g) with g(column) = the oracle value of that column."""
    cells, o = load_link()
    for r in range(1, 400):
        if f"A{r}" not in cells:
            break
        yield cells[f"A{r}"], cells[f"B{r}"], (lambda k, r=r: o.get(f"{k}{r}", {}).get("v"))
