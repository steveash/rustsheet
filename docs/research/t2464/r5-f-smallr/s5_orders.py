"""Chain orders with a free constant: terms C (free, scanned), P (stored power), H = -hL (extended) or its halves
Ha = -aL, Hb = -bL (each extended, or stored with a trailing 's'), summed left to right, each partial sum stored (1) or
extended (0).  Y_model = chain - P.  Usage: s5_orders.py SET[,SET] [chains]"""
import itertools
import sys

import numpy as np

import r5lib as R

LD = R.LD


def terms(A):
    h, a, b = A["h"], A["a"], A["b"]
    L = A["L"].astype(LD)
    return {"H": -LD(h) * L, "Ha": -LD(a) * L, "Hb": -LD(b) * L,
            "Has": -R.f64(LD(a) * L).astype(LD), "Hbs": -R.f64(LD(b) * L).astype(LD),
            "Hs": -R.f64(LD(h) * L).astype(LD), "P": A["P"].astype(LD)}


def ev(chain, C, t):
    names, flags = chain
    acc = C if names[0] == "C" else t[names[0]]
    for nm, fl in zip(names[1:], flags):
        acc = acc + (C if nm == "C" else t[nm])
        if fl:
            acc = R.f64(acc).astype(LD)
    return acc


def scan(A, t, chain, span=300.0, step=0.5):
    P = A["P"].astype(LD)
    Kh = LD(float(A["Kp"]))
    Klo = float((A["Kp"] - R.mpf(float(A["Kp"]))) * R.U)
    best = (0, 0.0)
    for d in np.arange(-span, span, step):
        C = Kh + LD(d + Klo) * LD(2.0) ** -53
        res = A["Y"] - R.units_rel(ev(chain, C, t) - P, A["Y0"])
        res = res - np.median(res)
        sc = int(np.sum(np.abs(res) <= 3))
        if sc > best[0]:
            best = (sc, d)
    return best


def all_chains():
    out = []
    for terms_ in (("C", "H", "P"), ("C", "Ha", "Hb", "P"), ("C", "Has", "Hbs", "P"), ("C", "Hs", "P")):
        for perm in itertools.permutations(terms_):
            for fl in itertools.product((0, 1), repeat=len(perm) - 1):
                if fl[-1] == 0 and len(perm) == 3:
                    pass
                out.append((perm, fl))
    return out


def name(ch):
    return "".join(n if n == "C" or n == "P" else n for n in ch[0]) + ":" + "".join(map(str, ch[1]))


if __name__ == "__main__":
    sets = sys.argv[1].split(",")
    chains = all_chains()
    tot = {name(c): 0 for c in chains}
    N = 0
    for src in sets:
        for blk, rs, m, n in R.runs(src, R.small_r):
            A = R.run_arrays(rs, m, n)
            t = terms(A)
            N += len(rs)
            sc = {name(c): scan(A, t, c)[0] for c in chains}
            for k, v in sc.items():
                tot[k] += v
            top = sorted(sc.items(), key=lambda kv: -kv[1])[:4]
            print(f"{src} {blk:26s} n={len(rs)} HKP11={sc['CHP:11']}", " ".join(f"{k}={v}" for k, v in top))
            sys.stdout.flush()
    print("TOTAL of", N)
    for k, v in sorted(tot.items(), key=lambda kv: -kv[1])[:25]:
        print(f"  {k:22s} {v}")
