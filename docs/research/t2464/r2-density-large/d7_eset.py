"""For each cell, the doubles E with EXP_V6(E) == Excel (scan +-6 ulps around ln(excel)); print them relative to the
model value C + X (in ulps of E) along the run, to see what the mismatches need."""
import math
import sys
import mpmath
import dlib as L
import loader as LD
import s2_scan as S

blk, spec = sys.argv[1], sys.argv[2]
best, *_ = S.run(blk, spec, quiet=True)
C = best[1]
kw = S.parse(spec)
kw.pop("comb", None)
rows = S.rows
idx = sorted(L.blocks(rows)[blk], key=lambda i: rows[i]["x"])[: int(sys.argv[3]) if len(sys.argv) > 3 else 60]
for i in idx:
    r = rows[i]
    X = LD.xpart(r["x"], r["a"], r["b"], **kw)[0]
    s = C + X
    e0 = float(mpmath.log(r["excel"]))
    u = math.ulp(e0)
    ok = [j for j in range(-8, 9) if L.exp_v6(mpmath.mpf(e0 + j * u)) == r["excel"]]
    pos = float((s - mpmath.mpf(e0)) / u)
    m = int(math.frexp(r["x"])[0] * 2**53) & 7
    print(f"{r['x']!r:24s} low3={m} model E at {pos:+6.2f} ulp; ok E offsets {ok}  {'HIT' if any(abs(pos - j) <= 0.5 for j in ok) else ''}")
