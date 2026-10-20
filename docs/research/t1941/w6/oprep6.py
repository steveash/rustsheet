# T-1941 W6: per-op repair test (W2's K2 finding: Excel keeps some Horner stages unstored, i.e. in 80-bit x87). For
# each op of the chain (product P_j / sum S_j of N or D, and the final division), simulate R with that one op left
# unstored (rounded to 64-bit only, not to double) and count, against the data's [Rlo, Rhi], points repaired (wrong ->
# right) and broken (right -> wrong). An op that Excel really keeps unstored repairs many more than it breaks.
# Usage: oprep6.py RFILE CHAIN [npts] [KEEP=N.S4,D.P3 (ops kept unstored in the base, comma list)]
import sys, numpy as np

sys.path.insert(0, sys.path[0])
from lib6 import st, L
from synth import read_chain

rows = [l.split() for l in open(sys.argv[1])]
npts = int(sys.argv[3]) if len(sys.argv) > 3 else 200000
idx = np.random.default_rng(0).choice(len(rows), min(npts, len(rows)), replace=False)
x = np.array([float.fromhex(rows[i][0]) for i in idx])
lo = np.array([float.fromhex(rows[i][2]) for i in idx])
hi = np.array([float.fromhex(rows[i][3]) for i in idx])
N, D = read_chain(sys.argv[2])
N = np.array(N) / D[0]
D = np.array(D) / D[0]
base_keep = set(sys.argv[4].split(",")) if len(sys.argv) > 4 and sys.argv[4] else set()


def chain(C, ch, keep):
    """raw-x Horner with x87 ops; ops named ch.Pj / ch.Sj in keep stay 64-bit (unstored)."""
    s = L(C[-1])
    for j in range(len(C) - 2, -1, -1):
        p = s * L(x)
        if "%s.P%d" % (ch, j) not in keep:
            p = L(st(p))
        s = p + L(C[j])
        if "%s.S%d" % (ch, j) not in keep:
            s = L(st(s))
    return s


def R_of(keep):
    q = chain(N, "N", keep) / chain(D, "D", keep)
    return st(q) if "Q" not in keep else st(q)


def ok(R):
    return (R >= lo) & (R <= hi)


base = ok(R_of(base_keep))
print("base exact %.4f (keep %s)" % (base.mean(), sorted(base_keep) or "none"))
ops = [
    "%s.%s%d" % (ch, k, j)
    for ch, C in (("N", N), ("D", D))
    for j in range(len(C) - 2, -1, -1)
    for k in ("P", "S")
]
res = []
for op in ops:
    if op in base_keep:
        continue
    t = ok(R_of(base_keep | {op}))
    rep, brk = int(np.sum(t & ~base)), int(np.sum(~t & base))
    res.append((rep - brk, rep, brk, op))
res.sort(reverse=True)
for net, rep, brk, op in res:
    print(
        "  %-6s repaired %6d  broken %6d  net %+6d  -> exact %.4f"
        % (op, rep, brk, net, (base.sum() + net) / len(base))
    )
