"""Free-constant scan of chain forms on one run: model Y = chain(C, H, P) - P + c2 with C scanned around K'
(step 0.25 unit over +-SPAN units) and c2 the median offset.  Prints the best C per form, score (# |res| <= 3) and
the residual sequence of the best.  Usage: s1_scan.py SRC tag [forms]"""
import sys

import numpy as np

import r5lib as R

LD = R.LD


def H_forms(A):
    h, a, b = A["h"], A["a"], A["b"]
    L = A["L"].astype(LD)
    return {
        "hL": -LD(h) * L,
        "fl(hL)": -R.f64(LD(h) * L).astype(LD),
        "fl(aL)+fl(bL)": -(R.f64(LD(a) * L).astype(LD) + R.f64(LD(b) * L).astype(LD)),
    }


def chain(form, C, H, P):
    if form == "HKP11":
        S1 = R.f64(C + H).astype(LD)
        return R.f64(S1 + P).astype(LD)
    if form == "HKP01":
        return R.f64(C + H + P).astype(LD)
    raise ValueError(form)


def scan(A, Hk, form, span=300.0, step=0.25):
    P = A["P"].astype(LD)
    Kh = LD(float(A["Kp"]))
    Klo = float((A["Kp"] - R.mpf(float(A["Kp"]))) * R.U)
    best = None
    for d in np.arange(-span, span, step):
        C = Kh + LD(d + Klo) * LD(2.0) ** -53
        Ym = R.units_rel(chain(form, C, Hk, P) - P, A["Y0"])
        res = A["Y"] - Ym
        c2 = np.median(res)
        sc = int(np.sum(np.abs(res - c2) <= 3))
        if best is None or sc > best[0]:
            best = (sc, d, c2, res - c2)
    return best


if __name__ == "__main__":
    src, tag = sys.argv[1], sys.argv[2]
    forms = sys.argv[3].split(",") if len(sys.argv) > 3 else ["HKP11", "HKP01"]
    for blk, rs, m, n in R.PL.runs(src, 0, tag):
        A = R.run_arrays(rs, m, n)
        for hn, Hk in H_forms(A).items():
            for f in forms:
                sc, d, c2, res = scan(A, Hk, f)
                print(f"{blk} {hn:14s} {f} {sc}/{len(res)} d={d:+.2f} c2={c2:+.1f}")
                if f == forms[0] and hn == "hL":
                    print("   res:", " ".join(f"{v:.0f}" for v in res))
