"""Excel's and the model's error against the exact density (units of 2^-53) on probe rows of a kind, with ln F, c ln x.
Usage: s6_vsexact.py KIND [d1 d2]"""
import math
import sys

import mpmath

import edgemodel as MD
import edgeprobe as MK
import s4_probe as S4

mpmath.mp.prec = 140
kind = sys.argv[1]
want = tuple(float(v) for v in sys.argv[2:4]) if len(sys.argv) > 3 else None
for o in MK.rows("a"):
    if o["kind"] != kind or (want and (o["d1"], o["d2"]) != want):
        continue
    e, x = S4.excel_of(o), S4.x_of(o)
    if not isinstance(e, float) or e == 0 or x <= 0:
        continue
    m, n = math.trunc(o["d1"]), math.trunc(o["d2"])
    X, M, N = mpmath.mpf(x), mpmath.mpf(m), mpmath.mpf(n)
    lv = (M / 2) * mpmath.log(M * X / (M * X + N)) + (N / 2) * mpmath.log(N / (M * X + N)) - mpmath.log(X) - mpmath.log(mpmath.beta(M / 2, N / 2))
    ex = mpmath.exp(lv)
    mv = MD.value(x, o["d1"], o["d2"])
    u = lambda v: float((mpmath.mpf(v) - ex) / ex * 2**53)  # noqa: E731
    side = 1 if m * x <= n else 2
    c = m / 2 if side == 1 else -n / 2
    print(f"{o['row']} x={x:.6g} ({m},{n}) s{side} lnF={float(lv):9.2f} clnx={c * math.log(x):9.2f} xl={u(e):8.1f} model={u(mv) if isinstance(mv, float) and mv else mv!s:>8}")
