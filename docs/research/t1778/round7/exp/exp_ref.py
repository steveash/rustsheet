"""Best EXP model (T-1778 round 4, exp-exact), portable (no x87 needed).
x87 sequence: FLDL2E; FMUL a (64-bit PC, RN) -> t; FRNDINT with RC=down -> n=floor(t); FSUB (RN) -> f=RN64(t-n);
F2XM1 -> u (modelled as RN64(2^f-1)); FLD1; FADD -> w=RN64(1+u); FSCALE -> w*2^n (exact); FST m64 (RN; subnormal
results kept, single rounding at the subnormal position).
Matches 153,643 of 153,655 normal-result EXP cells of kmyst-erfc-* (the 18 cutoff cells read 0.0 are kept subnormals,
which the Graph API reports as 0.0; numbers.md 'Subnormal results become 0')."""
import mpmath as mp
mp.mp.prec = 256
_L2E64 = None
def _r64(v):
    with mp.workprec(64):
        return +v
def excel_exp(a: float) -> float:
    global _L2E64
    if _L2E64 is None:
        _L2E64 = _r64(1 / mp.log(2))
    t = _r64(mp.mpf(a) * _L2E64)
    n = mp.floor(t)
    f = _r64(t - n)
    u = _r64(mp.power(2, f) - 1)
    w = _r64(1 + u)
    return float(mp.ldexp(w, int(n)))   # float(): RN to binary64, gradual underflow
if __name__ == '__main__':
    import sys
    ok = n = 0; bad = []
    for l in open(sys.argv[1]):
        ah, eh, tag = l.split(); a = float.fromhex(ah); E = float.fromhex(eh)
        if E == 0: continue
        n += 1; r = excel_exp(a)
        if r == E: ok += 1
        else: bad.append((ah, eh, r.hex(), tag))
    print(f'{ok}/{n}')
    for b in bad: print(*b)
    for x, v in [(2, 7.38905609893065), (1, 2.718281828459045), (-708, 3.307553003638408e-308)]:
        print('EXP(%g)' % x, excel_exp(x) == v)
    print('EXP(-745) =', excel_exp(-745), 'EXP(-720) =', excel_exp(-720))
