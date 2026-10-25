"""Jump attribution: positions k where tau = ln(Excel) + ln x - fl(c LN x) jumps by >= 16 units (an extra stored
term changing), against the positions where candidate stored doubles change, and the change sizes (units).
Usage: s11_jumps.py SET run-substring"""
import sys

import mpmath

import fblib as B

F = B.F
LDB, st = F.LDB, F.st
mpf = mpmath.mpf
U = mpf(2) ** 53
SET, pat = sys.argv[1], sys.argv[2]
rr = [r for r in B.load(SET) if pat in r["blk"]]
m, n = rr[0]["d1"], rr[0]["d2"]
a, b, h = m / 2, n / 2, (m + n) / 2


def cands(x):
    mx = st(LDB(m) * LDB(x))
    side = 1 if mx <= n else 2
    r = st(LDB(mx) / LDB(n)) if side == 1 else st(LDB(n) / LDB(mx))
    s = st(LDB(1) + LDB(r))
    L = float(F.TD.log1p_xl(r))
    big = st(LDB(1) / LDB(s))
    small = st(LDB(r) / LDB(s))
    c = a if side == 1 else -b
    out = {
        "tau": None,
        "r": r, "s": s, "1/s": big, "r/s": small,
        "t=r/(r+2)": st(LDB(r) / LDB(st(LDB(r) + LDB(2)))),
        "r+2": st(LDB(r) + LDB(2)),
        "fl(hL)": st(LDB(h) * LDB(L)),
        "fl(aL)": st(LDB(a) * LDB(L)),
        "fl(bL)": st(LDB(b) * LDB(L)),
        "a*LN(small)": st(LDB(a if side == 1 else b) * F.LN(small)),
        "b*LN(big)": st(LDB(b if side == 1 else a) * F.LN(big)),
        "c*LN(x)": st(LDB(c) * F.LN(x)),
        "LN(x)": float(F.LN(x)),
    }
    tau = mpmath.log(rr_ex[x]) + mpmath.log(x) - mpf(out["c*LN(x)"])
    out["tau"] = tau
    return out


rr_ex = {r["x"]: r["excel"] for r in rr}
seq = [cands(r["x"]) for r in rr]
ex = [r["exact"] for r in rr]
taus = [c["tau"] for c in seq]
jumps = [k for k in range(1, len(taus)) if abs((taus[k] - taus[k - 1]) * U) >= 16]
print(f"{rr[0]['blk']}: tau jumps at", {k: round(float((taus[k] - taus[k - 1]) * U)) for k in jumps})
for name in seq[0]:
    if name == "tau":
        continue
    ch = [k for k in range(1, len(seq)) if seq[k][name] != seq[k - 1][name]]
    sz = {k: round(float((mpf(seq[k][name]) - mpf(seq[k - 1][name])) * U), 1) for k in ch[:12]}
    hit = len(set(ch) & set(jumps))
    print(f"  {name:12s} changes {len(ch):2d}, at tau jumps {hit:2d}/{len(jumps)}:", sz)
