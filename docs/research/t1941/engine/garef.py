"""GAMMA reference composed from the T-1941 checkers (mpmath exact x87 rounding: fl = RN53(RN64(.))).
Pieces (README "GAMMA ..."): integers <= 88 downward product; exact half-integers (gamma_neg_check.py); x >= 10 binary
powering (gamma_big_check.c); 1 <= x < 10 descending product * G; 0 < x < 1; -10 <= x < 0 division chain; x < -10
reflection with FSIN = RN64(sin); G = K1 plain1 (w4/k1r/plain1.txt), stored raw-y Horner (7,7), fl(N/D)."""
import sys, math, mpmath as mp
S = sys.argv[1]; sys.path.insert(0, S)
mp.mp.prec = 200
from exp_v6 import excel_exp
STORE = sys.argv[2] if len(sys.argv) > 2 else "x87"   # "x87" = RN53(RN64) per op, "sse" = RN53 once
def rbits(v, bits):
    if v == 0: return mp.mpf(0)
    s, man, exp, bc = v._mpf_
    if bc <= bits: return v
    sh = bc - bits; q = man >> sh; r = man - (q << sh); half = 1 << (sh - 1)
    if r > half or (r == half and q & 1): q += 1
    return mp.mpf((s, q, exp + sh, q.bit_length()))
def fl(v):
    v = mp.mpf(v)
    return float(rbits(rbits(v, 64), 53)) if STORE == "x87" else float(rbits(v, 53))
def M(a, b): return fl(mp.mpf(a) * mp.mpf(b))
def A(a, b): return fl(mp.mpf(a) + mp.mpf(b))
def Dv(a, b): return fl(mp.mpf(a) / mp.mpf(b))
_L2 = rbits(mp.log(2), 64)
def LN(x): return float(rbits(rbits(_L2 * mp.log(mp.mpf(x), 2), 64), 53))
H = lambda s: [float.fromhex(h) for h in s.split()]
K1 = {l.split()[0]: H(' '.join(l.split()[1:])) for l in open(S + '/plain1.txt') if l.strip() and not l.startswith('#')}
def horner(C, y):
    s = C[-1]
    for c in reversed(C[:-1]): s = A(M(s, y), c)
    return s
def G(y): return Dv(horner(K1['N'], y), horner(K1['D'], y))
PI = float(mp.pi); SQRTPI = float(mp.sqrt(mp.pi))
B = [1.0/12, -1.0/360, 1.0/1260, -1.0/1680, 1.0/1188, -691.0/360360, 1.0/156, -3617.0/122400, 43867.0/244188, -174611.0/125400]
def big(x):
    h = 0.5 * x; nn = math.floor(h); m = int(nn); c2 = h - nn
    r = 1.0; bb = x; first = True
    while m:
        if m & 1:
            r = bb if first else M(r, bb); first = False
        m >>= 1
        if m: bb = M(bb, bb)
    v = M(r, excel_exp(M(c2, LN(x))))
    w1 = Dv(1.0, x); z2 = M(w1, w1); rr = B[9]
    for k in range(8, -1, -1): rr = A(M(rr, z2), B[k])
    W = excel_exp(M(rr, w1))
    sq = fl(mp.sqrt(mp.mpf(w1)))
    t = M(excel_exp(-x), sq); t = M(t, v); t = M(t, v); t = M(t, 2.50662827463100050242); return M(t, W)
def pos_half(x):
    p = 1.0; k = 0.5
    while k < x: p = M(p, k); k += 1
    return M(p, SQRTPI)
def flush(v): return 0.0 if abs(v) < 2.2250738585072014e-308 else v
def sign_neg(x): return -1.0 if math.ceil(-x) % 2 else 1.0
def gamma(x):
    if not (x == x) or x in (math.inf, -math.inf): return math.nan
    if x == math.floor(x):
        if x <= 0: return math.nan
        if x <= 88:
            p = 1.0
            for k in range(int(x) - 1, 1, -1): p = M(p, k)
            return p
    if x == math.floor(x) + 0.5:
        if x > 0: return pos_half(x)
        q = -x; g = pos_half(q)
        return sign_neg(x) * flush(Dv(Dv(PI, g), q)) + 0.0
    if x >= 10: return big(x)
    if x >= 1:
        y = x; z = 1.0
        while y >= 2: y = y - 1.0; z = M(z, y)
        return M(z, G(y))
    if x > 0: return M(Dv(1.0, x), G(A(x, 1.0)))
    if -x <= 10:
        z = Dv(1.0, x); y = A(x, 1.0)
        while y < 1: z = Dv(z, y); y = A(y, 1.0)
        return M(z, G(y))
    q = -x; p = math.floor(q); zz = q - p
    if zz > 0.5: p += 1; zz = q - p
    s = abs(fl(mp.sin(mp.mpf(M(PI, zz)))))
    gq = gamma(q)
    if gq == math.inf or gq != gq: return 0.0
    return sign_neg(x) * flush(Dv(Dv(PI, gq), M(q, s))) + 0.0
