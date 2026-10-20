import sys, mpmath, struct, random
sys.path.insert(0, sys.argv[1]); from rnd import rn53, rn64
mpmath.mp.prec = 300
L2 = rn64(mpmath.log(2))
def model(x): return rn53(rn64(L2 * mpmath.log(mpmath.mpf(x), 2)))
bits = lambda f: format(struct.unpack('<Q', struct.pack('<d', f))[0], '016x')
seen = {}
for l in open(sys.argv[2]):
    a, b, c = l.split(); seen.setdefault(a, b)
keys = sorted(seen)
random.seed(2123)
sample = set(random.sample(keys, 4000))
misses = [k for k in keys if model(float.fromhex(k)) != float.fromhex(seen[k])] if len(sys.argv) > 4 else []
out = open(sys.argv[3], 'w')
out.write("# T-2123 LN fixture: x excel model (f64 bits). 4,000 random distinct x from the T-1941 kmyst-gam-*\n")
out.write("# LN cells plus every point where Excel differs from the model; model = lnmodels.py's FYL2X model.\n")
for k in sorted(sample | set(misses), key=float.fromhex):
    x = float.fromhex(k); out.write(f"{bits(x)} {bits(float.fromhex(seen[k]))} {bits(model(x))}\n")
