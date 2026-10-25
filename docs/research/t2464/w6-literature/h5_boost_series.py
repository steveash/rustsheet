"""w6-literature H5: Boost.Math's ibeta_series term scaling in Excel's bpser branch.

Boost (include/boost/math/special_functions/beta.hpp, ibeta_series + ibeta_series_t) multiplies the prefix into
the running term BEFORE summing:  t_0 = P;  term_n = t_n / (a+n);  t_{n+1} = t_n * ((poch * x) / (n+1)),
poch = n+1-b;  S = sum term_n (forward, tools::sum_series: stop after a term with |term| <= eps |S|).
w3's model sums s = 1/a + sum c_n/(a+n) unscaled and multiplies by P at the end (I = EXP(z) * s).
Both use the same P = EXP(z), z = fl53(fl(a LN x) - L) (w3 model, z53). Scored on the TOMS bratio bpser
branches (w = bpser(a0, b0, x0), no reflection) of every cdf cell, with the md5 hold-out half.
"""

import collections
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent / "w3copy"))
from dispatch import branch  # noqa: E402
from lib import add, div, exact_cdf, fmt, load, mul, mpmath, np, save_exact, score, split_half, sub  # noqa: E402
from model import exp_of, exponent, model  # noqa: E402


def boost_sum(x, a, b, P, stop="eps", nmax=20000):
    t = P.copy()
    S = div(t, a)
    poch = sub(1.0, b)
    n = np.ones_like(x)
    active = np.ones(x.shape, dtype=bool)
    for _ in range(nmax):
        t_new = mul(t, div(mul(poch, x), n))
        apn = add(a, n)
        term = div(t_new, apn)
        Sn = add(S, term)
        done = (Sn == S) if stop == "unchanged" else (np.abs(term) <= 2.220446049250313e-16 * np.abs(Sn))
        t = np.where(active, t_new, t)
        S = np.where(active, Sn, S)
        active &= ~done
        if not active.any():
            break
        poch = add(poch, 1.0)
        n = add(n, 1.0)
    return S


def main():
    rows = [r for r in load(cum=True) if r["a"] != 1.0 and r["b"] != 1.0 and max(r["a"], r["b"]) <= 1e4]
    sel = []
    for r in rows:
        y = float(sub(1.0, r["x"]))
        br = branch(r["a"], r["b"], r["x"], y)
        if br[0] == "w_bpser" and not br[5]:
            sel.append((r, br))
    rs = [r for r, _ in sel]
    E = np.array([r["excel"] for r in rs])
    cr = np.array([float(exact_cdf(r["x"], r["a"], r["b"])) for r in rs])
    save_exact()
    xx = np.array([br[3] for _, br in sel])
    aa = np.array([br[1] for _, br in sel])
    bb = np.array([br[2] for _, br in sel])
    w3 = model(xx, aa, bb, big=8.0, zmode="z53")
    z = exponent(xx, aa, bb, big=8.0)
    P = np.asarray(exp_of(z, "z53"), dtype=np.float64)
    bst = boost_sum(xx, aa, bb, P, "eps")
    bstu = boost_sum(xx, aa, bb, P, "unchanged")
    tune, hold = split_half(rs)
    hid = {id(r) for r in hold}
    groups = collections.defaultdict(list)
    for i, r in enumerate(rs):
        groups["min<=1" if min(r["a"], r["b"]) <= 1 else "a,b>1 (b*x<=0.7)"].append(i)
        groups["block " + r["block"].split(":")[0]].append(i)
        groups[f"README branch {r['branch']} ({r['src']})"].append(i)
    for name, m in [
        ("CR", cr),
        ("w3 model (EXP(z)*s)", w3),
        ("Boost-scaled, eps stop", bst),
        ("Boost-scaled, unchanged stop", bstu),
    ]:
        print(f"{name:30s} all {fmt(score(m, E))}")
        for g in sorted(groups):
            idx = np.array(groups[g])
            ih = np.array([i for i in idx if id(rs[i]) in hid])
            print(
                f"    {g:24s} {fmt(score(m[idx], E[idx])):24s} hold-out {fmt(score(m[ih], E[ih])) if len(ih) else '-'}"
            )


if __name__ == "__main__":
    main()
