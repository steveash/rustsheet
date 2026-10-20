import ctypes, numpy as np, os
K3 = os.path.dirname(os.path.abspath(__file__)) + '/'
_l = ctypes.CDLL(K3 + 'libk3s.so'); DP = ctypes.POINTER(ctypes.c_double)
_l.score.restype = ctypes.c_long
_l.score.argtypes = [DP, DP, ctypes.c_long, DP, ctypes.c_int, DP, ctypes.c_int] + [ctypes.c_int, ctypes.c_int, ctypes.c_double, ctypes.c_uint, ctypes.c_uint] * 2
_l.resv.argtypes = [DP, ctypes.c_long, DP, ctypes.c_int, DP, ctypes.c_int] + [ctypes.c_int, ctypes.c_int, ctypes.c_double, ctypes.c_uint, ctypes.c_uint] * 2 + [DP]
def _p(a): return a.ctypes.data_as(DP)
def _c(a): return np.ascontiguousarray(a, dtype=np.float64)
NOPT = (-1, 0, 0.0, 0, 0)
def score(x, b, p, q, oN=NOPT, oD=NOPT):
    p = _c(p); q = _c(q); return _l.score(_p(x), _p(b), len(x), _p(p), len(p), _p(q), len(q), *oN, *oD) / len(x)
def resv(x, p, q, oN=NOPT, oD=NOPT):
    p = _c(p); q = _c(q); o = np.empty_like(x); _l.resv(_p(x), len(x), _p(p), len(p), _p(q), len(q), *oN, *oD, _p(o)); return o
def ldset(f):
    l = open(f).read().split('\n'); return [float.fromhex(v) for v in l[0].split()], [float.fromhex(v) for v in l[1].split()]
def nd(v, k):
    i = np.frombuffer(np.float64(v).tobytes(), np.int64)[0] + (k if v >= 0 else -k); return float(np.frombuffer(np.int64(i).tobytes(), np.float64)[0])
def rows(sub=None, seed=0, path=os.environ.get('K3ROWS', 'rows.npz')):   # from w5/p5/mkrows.py
    z = np.load(path); x, b, c = z['x'], z['b'], z['c']
    ok = ~np.isin(c, ['kmyst-gam-q501', 'kmyst-gam-q502']); x, b, c = x[ok], b[ok], c[ok]
    if sub: i = np.random.default_rng(seed).choice(len(x), sub, replace=False); i.sort(); x, b, c = x[i], b[i], c[i]
    return _c(x), _c(b), c
