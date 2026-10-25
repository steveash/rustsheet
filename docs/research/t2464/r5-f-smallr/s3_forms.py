"""Scan forms of H = -h L(r) under the HKP11 chain with a free constant per run (s1_scan.scan), summed over the
informative small-r runs of the given sets.  Forms: r (how the ratio is formed) x L (which log1p) x H (how h L is
stored).  Usage: s3_forms.py SET[,SET...] [rforms] [lforms] [hforms] [chain]"""
import sys

import mpmath
import numpy as np

import r5lib as R
import s1_scan as S

LD, st, LDB = R.LD, R.st, R.LDB
XL = R.F.TD


def r_of(x, m, n, side1, form):
    M, N, X = LDB(m), LDB(n), LDB(x)
    mx = st(M * X)
    if form == "mx":
        return st(LDB(mx) / N) if side1 else st(N / LDB(mx))
    if form == "e":
        return st(M * X / N) if side1 else st(N / (M * X))
    if form == "q":
        return st(LDB(st(M / N)) * X) if side1 else st(LDB(st(N / M)) / X)
    if form == "inv":
        return st(LDB(mx) / N) if side1 else st(LDB(1) / LDB(st(LDB(mx) / N)))
    if form == "xm":  # side 2 as fl(fl(n/x)/m)
        return st(LDB(mx) / N) if side1 else st(LDB(st(N / X)) / M)
    raise ValueError(form)


def L_of(r, form):
    if form == "xl":
        return LD(XL.log1p_xl(r))
    if form == "exact":
        return R.F.R3._ld_of(mpmath.log1p(mpmath.mpf(r)))
    if form == "ln":
        return LD(R.F.LN(st(LDB(1) + LDB(r))))
    if form == "l1mz":  # -log1p_xl(-z), z = r/(1+r)
        z = st(LDB(r) / LDB(st(LDB(1) + LDB(r))))
        return -LD(XL.log1p_xl(-z))
    if form == "exr":  # exact log1p of the exact ratio (no r rounding at all): handled by caller
        raise ValueError
    raise ValueError(form)


def H_arr(rs, m, n, side1, rf, lf, hf):
    a, b, h = m / 2, n / 2, (m + n) / 2
    out = []
    for q in rs:
        x = q["x"]
        if lf == "exr":
            X = mpmath.mpf(x)
            L = R.F.R3._ld_of(mpmath.log1p(m * X / n if side1 else n / (m * X)))
        else:
            L = L_of(r_of(x, m, n, side1, rf), lf)
        if hf == "ext":
            v = -LD(h) * L
        elif hf == "st":
            v = -LD(st(LD(h) * L))
        elif hf == "ab":
            v = -(LD(st(LD(a) * L)) + LD(st(LD(b) * L)))
        elif hf == "abst":
            v = -LD(st(LD(st(LD(a) * L)) + LD(st(LD(b) * L))))
        else:
            raise ValueError(hf)
        out.append(v)
    return np.array(out, dtype=LD)


def main():
    sets = sys.argv[1].split(",")
    rfs = (sys.argv[2] if len(sys.argv) > 2 else "mx,e,q,inv,xm").split(",")
    lfs = (sys.argv[3] if len(sys.argv) > 3 else "xl,exact,ln,l1mz,exr").split(",")
    hfs = (sys.argv[4] if len(sys.argv) > 4 else "ext,st,ab").split(",")
    ch = sys.argv[5] if len(sys.argv) > 5 else "HKP11"
    tot: dict = {}
    N = 0
    for src in sets:
        for blk, rs, m, n in R.runs(src, R.small_r):
            A = R.run_arrays(rs, m, n)
            N += len(rs)
            line = []
            for rf in rfs:
                for lf in lfs:
                    if lf == "exr" and rf != rfs[0]:
                        continue
                    for hf in hfs:
                        Hk = H_arr(rs, m, n, A["side1"], rf, lf, hf)
                        sc = S.scan(A, Hk, ch)[0]
                        key = f"{rf}/{lf}/{hf}"
                        tot[key] = tot.get(key, 0) + sc
                        line.append(sc)
            print(f"{src} {blk:28s} n={len(rs)}", " ".join(map(str, line)))
            sys.stdout.flush()
    print("TOTAL of", N)
    for k, v in sorted(tot.items(), key=lambda t: -t[1]):
        print(f"  {k:20s} {v}")


if __name__ == "__main__":
    main()
