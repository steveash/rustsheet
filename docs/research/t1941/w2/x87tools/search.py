"""ulp-step helpers and null-direction segment search (moves c[off..off+k] of one polynomial along the direction that
keeps the polynomial value ~unchanged on [2,3], in ulp units of each coefficient)."""
import math, struct
import numpy as np
def nextd(v, k):
    i = struct.unpack('<q', struct.pack('<d', v))[0]; i += k if v >= 0 else -k
    return struct.unpack('<d', struct.pack('<q', i))[0]
def null_dir(c, k, xg=np.linspace(2, 3, 20)):
    u = [math.ulp(abs(v)) for v in c[:k + 1]]
    A = np.array([[x ** (k - i) * u[i] for i in range(1, k + 1)] for x in xg]); b = -np.array([x ** k * u[0] for x in xg])
    return np.concatenate([[1.0], np.linalg.lstsq(A, b, rcond=None)[0]])
def dsearch(p, q, sc, which, k, T, step=1, off=0):
    c = list(p if which == 'p' else q); d = null_dir(c[off:off + k + 1], k); best = (sc(p, q), 0, None)
    for t in range(-T, T + 1, step):
        cc = list(c)
        for i, di in enumerate(d): cc[off + i] = nextd(c[off + i], int(round(t * di)))
        s = sc(cc, q) if which == 'p' else sc(p, cc)
        if s > best[0]: best = (s, t, cc)
    if best[2] is None: return p, q, best[:2]
    return (best[2], q, best[:2]) if which == 'p' else (p, best[2], best[:2])
