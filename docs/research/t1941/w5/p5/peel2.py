"""Top-down peeling with the exact two-site hypothesis template (T-1941 W5 part 5; fixes peel.py's tie scrambling).
usage: peel2.py control|real CHAIN J [OUT]   env: BASE, TRUTHFILE (control), KMAX, KSTEP, APPLY"""
import numpy as np, sys, os, ctypes, time
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, '..'))
from sc import res
from meter import sites_and_weights, resid, L
_lib = ctypes.CDLL(os.path.join(HERE, 'libpeelk.so')); Pd = ctypes.POINTER(ctypes.c_double); Pl = ctypes.POINTER(ctypes.c_long)
_lib.peel_obj.argtypes = [Pd, Pd, Pd, Pd, Pd, Pl, ctypes.c_int, ctypes.c_double, ctypes.c_double, Pl, ctypes.c_int, Pd]
mode, CH, J = sys.argv[1], sys.argv[2], int(sys.argv[3]); OUT = sys.argv[4] if len(sys.argv) > 4 else None
def ld(f):
    l = open(f).read().split('\n'); return np.array([float.fromhex(v) for v in l[0].split()]), np.array([float.fromhex(v) for v in l[1].split()])
N0, D0 = ld(os.environ.get('BASE', os.path.join(HERE, '..', 'p4', 'ss_fc.txt')))
ROWS = os.environ.get('ROWS', '/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows2.txt')
z = np.load(ROWS+'.near1.npz'); x = z['x']; b = z['b']
LO, HI = float(os.environ.get('LO', 0.7)), float(os.environ.get('HI', 1.47)); RL = int(os.environ.get('RL', 4096))
br = np.where(np.nextafter(x[:-1], np.inf) != x[1:])[0]+1
segs = [s[:RL] for s in np.split(np.arange(len(x)), br) if len(s) >= 256 and LO <= x[s[0]] < HI and abs(x[s[0]]-1) > 2e-3]
if mode == 'control': Nt, Dt = ld(os.environ['TRUTHFILE'])
C0 = N0 if CH == 'N' else D0; uc = float(np.spacing(abs(C0[J])))
PJ = []; XX = []; WS = []; WP = []; RR = []; rs = [0]
for s in segs:
    xx = x[s]; W, ref = sites_and_weights(N0, D0, xx)
    r = resid(xx, res(xx, Nt, Dt) if mode == 'control' else b[s], ref)
    PJ.append(W[(CH, 'P', J)][1]); XX.append(xx); WS.append(W[(CH, 'S', J)][2].astype(float)*2.0**52); WP.append(W[(CH, 'P', J-1)][2].astype(float)*2.0**52)
    RR.append(r); rs.append(rs[-1]+len(xx))
cat = lambda a: np.ascontiguousarray(np.concatenate(a), dtype=np.float64)
PJ, XX, WS, WP, RR = map(cat, (PJ, XX, WS, WP, RR)); rs = np.array(rs, dtype=np.int64)
KMAX = int(float(os.environ.get('KMAX', 1e5))); KSTEP = int(os.environ.get('KSTEP', 1))
ks = np.arange(-KMAX, KMAX+1, KSTEP, dtype=np.int64); out = np.empty(len(ks))
t0 = time.time()
_lib.peel_obj(PJ.ctypes.data_as(Pd), XX.ctypes.data_as(Pd), WS.ctypes.data_as(Pd), WP.ctypes.data_as(Pd), RR.ctypes.data_as(Pd), rs.ctypes.data_as(Pl), len(segs), float(C0[J]), uc, ks.ctypes.data_as(Pl), len(ks), out.ctypes.data_as(Pd))
mu, sd = np.median(out), np.std(out); o = np.argsort(out)[::-1]
print('%s.%d: runs %d, %d hypotheses in %.0fs; top: %s' % (CH, J, len(segs), len(ks), time.time()-t0, ' | '.join('k=%+d %.1f (Z %.1f)' % (ks[i], out[i], (out[i]-mu)/sd) for i in o[:5])), flush=True)
if mode == 'control':
    Ct = Nt if CH == 'N' else Dt; kt = int(round((Ct[J]-C0[J])/uc)); i = np.where(ks == kt)[0]
    print('   truth k=%+d: %s' % (kt, ('%.1f (Z %.1f, rank %d)' % (out[i[0]], (out[i[0]]-mu)/sd, int(np.where(o == i[0])[0][0]))) if len(i) else 'not on grid'))
if OUT and os.environ.get('APPLY'):
    k = int(ks[o[0]]); N1 = N0.copy(); D1 = D0.copy(); C1 = N1 if CH == 'N' else D1; C1[J] += k*uc
    READ = {('N', 0), ('N', 1), ('N', 2), ('D', 1), ('D', 2), ('D', 3)}
    free = [(CH, i) for i in range(J) if (CH, i) not in READ] + [(o2, i) for o2 in ('N', 'D') if o2 != CH for i in range(1, J) if (o2, i) not in READ]
    xs = np.linspace(0.7, 1.5, 120).astype(L); tgt = np.polyval(N0.astype(L)[::-1], xs)/np.polyval(D0.astype(L)[::-1], xs)
    for it in range(3):
        Nv = np.polyval(N1.astype(L)[::-1], xs); Dv = np.polyval(D1.astype(L)[::-1], xs)
        M = np.array([(np.spacing(abs((N1 if c == 'N' else D1)[i]))*xs**i/(Nv if c == 'N' else -Dv)).astype(float) for c, i in free]).T
        sol, *_ = np.linalg.lstsq(M/1e-18, ((tgt*Dv/Nv)-1).astype(float)/1e-18, rcond=None)
        for (c, i), v in zip(free, sol):
            CC = N1 if c == 'N' else D1; CC[i] += round(v)*np.spacing(abs(CC[i]))
    open(OUT, 'w').write(' '.join(v.hex() for v in N1)+'\n'+' '.join(v.hex() for v in D1)+'\n'); print('   applied k=%+d -> %s' % (k, OUT))
