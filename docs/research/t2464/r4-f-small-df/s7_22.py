"""(2,2): F = 1/(1+x)^2 exactly. Candidate chains, exact-match counts per side on probe mya."""
import slib as L

LDB, st, LN, EXP = L.LDB, L.st, L.LN, L.EXP
rows = [r for r in L.load("mya") if (r["d1"], r["d2"]) == (2.0, 2.0)]


def cands(x):
    s, r = L.side_r(x, 2.0, 2.0)
    X = LDB(x)
    p = st(LDB(1) + X)
    Lr = L.L_xl(r)
    out = {
        "1/(p*p)": st(LDB(1) / LDB(st(LDB(p) * LDB(p)))),
        "(1/p)^2": st(LDB(st(LDB(1) / LDB(p))) ** 2),
        "1/p/p": st(LDB(st(LDB(1) / LDB(p))) / LDB(p)),
        "EXP(-2 LN p)": EXP(LDB(-2) * LN(p)),
        "EXP(-2 L(r)) [s2: /x^2]": EXP(LDB(-2) * Lr) if s == 1 else st(LDB(EXP(LDB(-2) * Lr - 2 * LN(x)))),
        "EXP(-2 L(r) - 2LN x)": EXP(LDB(-2) * Lr - (2 * LN(x) if s == 2 else 0)),
        "EXP(-2L)/x/x": EXP(LDB(-2) * Lr) if s == 1 else st(LDB(st(LDB(EXP(LDB(-2) * Lr)) / X)) / X),
        "EXP(-2L)/fl(x*x)": EXP(LDB(-2) * Lr) if s == 1 else st(LDB(EXP(LDB(-2) * Lr)) / LDB(st(X * X))),
        "EXP(-2L - fl(2LNx))": EXP(LDB(-2) * Lr - (LDB(st(2 * LN(x))) if s == 2 else 0)),
        "EXP(fl(-2L))": EXP(LDB(st(LDB(-2) * Lr))) if s == 1 else EXP(LDB(st(LDB(-2) * Lr)) - 2 * LN(x)),
        "M-xd": L.mxd(dict(x=x, d1=2.0, d2=2.0)),
        "EXP(-LNx-2L)/x s2": EXP(LDB(-2) * Lr) if s == 1 else st(LDB(EXP(-LN(x) - LDB(2) * Lr)) / X),
    }
    return s, out


tot: dict = {}
for r in rows:
    s, c = cands(r["x"])
    for k, v in c.items():
        t = tot.setdefault((k, s), [0, 0])
        t[0] += v == r["excel"]
        t[1] += 1
    t = tot.setdefault(("CR", s), [0, 0])
    t[0] += r["cr"] == r["excel"]
    t[1] += 1
for (k, s), (a, n) in sorted(tot.items(), key=lambda kv: (kv[0][1], kv[0][0])):
    print(f"side{s} {k:28s} {a:3d} of {n}")
