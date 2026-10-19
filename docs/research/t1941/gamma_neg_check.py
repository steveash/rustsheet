"""T-1941 (W3): GAMMA at half-integers (both signs) and the negative reflection, exact x87 rounding model.

Usage (repo root): python3 gamma_neg_check.py half.txt h102.txt h103.txt
  half.txt: "x_hex gamma_hex ..." (positive half-integers); h102.txt: "x_hex D_hex"; h103.txt: "x_hex D_hex E_hex ...".
Every op is fl53(fl64(.)) (RN64 then RN53, ties-to-even). FSIN modelled as RN64(sin(arg)).
"""
import sys, mpmath as mp
mp.mp.prec = 200

def rbits(v, bits):
    if v == 0: return mp.mpf(0)
    s, man, exp, bc = v._mpf_
    if bc <= bits: return v
    sh = bc - bits; q = man >> sh; r = man - (q << sh); half = 1 << (sh - 1)
    if r > half or (r == half and q & 1): q += 1
    return mp.mpf((s, q, exp + sh, q.bit_length()))

def fl(v): return mp.mpf(float(rbits(rbits(v, 64), 53)))
PI = mp.mpf(float(mp.pi)); SQRTPI = mp.mpf(float(mp.sqrt(mp.pi)))

def gamma_pos_half(x):
    """x = m + 0.5 > 0 exactly: ascending stored product 0.5*1.5*...*(x-1), then * sqrt(pi)."""
    p = mp.mpf(1); k = mp.mpf(0.5)
    while k < x: p = fl(p * k); k += 1
    return fl(p * SQRTPI)

def sign_gamma_neg(x):  # sign of Gamma(x) for x < 0 non-integer: (-1)^ceil(-x)
    return -1 if int(mp.ceil(-x)) % 2 else 1

def gamma_neg_half(x):
    """x = -(m + 0.5): +-fl(fl(PI/Gh(q))/q), q = -x."""
    q = -x
    return sign_gamma_neg(x) * fl(fl(PI / gamma_pos_half(q)) / q)

def gamma_reflect(x, gq):
    """x <= -30 (cut still being located): gq = Excel GAMMA(-x)."""
    q = -x; p = mp.floor(q); z = q - p
    if z > 0.5: p += 1; z = q - p
    s = abs(fl(mp.sin(fl(PI * z))))
    return sign_gamma_neg(x) * fl(fl(PI / gq) / fl(q * s))

def rows(fn): return [l.split() for l in open(fn)]

if __name__ == "__main__":
    ok = n = 0
    for r in rows(sys.argv[1]):
        x = mp.mpf(float.fromhex(r[0]))
        if x > 0: n += 1; ok += float(gamma_pos_half(x)) == float.fromhex(r[1])
    print(f"positive half-integers {ok}/{n}")
    ok = n = 0
    for r in rows(sys.argv[2]):
        x = mp.mpf(float.fromhex(r[0])); g = float.fromhex(r[1])
        if 2 * x != mp.floor(2 * x) or (2 * x) % 2 == 0 or g == 0: continue
        n += 1; ok += float(gamma_neg_half(x)) == g
    print(f"negative half-integers {ok}/{n}")
    ok = n = 0
    for r in rows(sys.argv[3]):
        if r[1] == 'nan' or r[2] == 'nan': continue
        x = mp.mpf(float.fromhex(r[0]))
        if x > -30: continue
        n += 1; ok += float(gamma_reflect(x, mp.mpf(float.fromhex(r[2])))) == float.fromhex(r[1])
    print(f"reflection x <= -30 {ok}/{n}")
