"""(2,2): the beta density is 1, so a beta-density-times-Jacobian implementation reduces to arithmetic on
y = m x/(m x + n), q = n/(m x + n) (or w5's forms). Exact-match counts of arithmetic candidates per side."""
import slib as L

LDB, st = L.LDB, L.st
rows = [r for r in L.load("mya") if (r["d1"], r["d2"]) == (2.0, 2.0)]


def o(a, op, b):
    a, b = LDB(a), LDB(b)
    return st({"+": a + b, "-": a - b, "*": a * b, "/": a / b}[op])


def cands(x):
    m, n = 2.0, 2.0
    mx = o(m, "*", x)
    d = o(mx, "+", n)
    y, q = o(mx, "/", d), o(n, "/", d)
    yw = o(1, "/", o(1, "+", o(n, "/", mx)))  # w5
    qw = o(1, "/", o(1, "+", o(mx, "/", n)))
    q1 = o(1, "-", y)
    p1 = o(1, "+", x)
    c = {}
    for nm, (Y, Q) in {"div": (y, q), "w5": (yw, qw), "1-y": (y, q1), "w5,1-y": (yw, o(1, "-", yw))}.items():
        c[f"{nm}: (yq)/x"] = o(o(Y, "*", Q), "/", x)
        c[f"{nm}: y(q/x)"] = o(Y, "*", o(Q, "/", x))
        c[f"{nm}: q(y/x)"] = o(Q, "*", o(Y, "/", x))
        c[f"{nm}: (y/x)q"] = o(o(Y, "/", x), "*", Q)
        c[f"{nm}: yq*(m/n)/x..."] = o(o(o(Y, "*", Q), "/", x), "*", 1.0)
        c[f"{nm}: q*q"] = o(Q, "*", Q)
        c[f"{nm}: (m/n) q^2"] = o(o(m / n, "*", Q), "*", Q)
    c["1/(1+x)/(1+x)"] = o(o(1, "/", p1), "/", p1)
    c["1/((1+x)(1+x))"] = o(1, "/", o(p1, "*", p1))
    c["4/(d*d)"] = o(4, "/", o(d, "*", d))
    c["(2/d)^2"] = o(o(2, "/", d), "*", o(2, "/", d))
    c["m n/(d d)"] = o(o(m * n, "/", d), "/", d)
    return c


tot: dict = {}
for r in rows:
    s, _ = L.side_r(r["x"], 2.0, 2.0)
    for k, v in cands(r["x"]).items():
        tot[(s, k)] = tot.get((s, k), 0) + (v == r["excel"])
for s in (1, 2):
    best = sorted(((v, k) for (ss, k), v in tot.items() if ss == s), reverse=True)[:8]
    print("side", s, best)
