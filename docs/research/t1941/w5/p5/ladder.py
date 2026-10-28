"""K3 ladder unwrap (T-1941 W5 part 5). Per anchor: the base run (kk = 00) and runs at anchor + 2^-kk (kk = 10..21),
each 8192 consecutive doubles. For each readable stage the line phase (phaselib.read_chunk) of every run, relative to
the base run, is unwrapped from the smallest offset to the largest: the local slope s_j(anchor) of the pre-rounding
offset delta_j (cycles of ulp(S_j) per unit x). Control: B synthesised from TRUTHFILE (or make_truth TSIZE) at the
ladder x; the true slope is d/dx of (S_j,truth - S_j,base)/ulp(S_j) from the smooth stage polynomials.
usage: ladder.py SET control|real   env: TRUTHFILE | TSIZE, SEED, STAGES, ANCH, KMIN (smallest kk used), QUAD, MINRUNG, TOL"""
import numpy as np, sys, os, json, math
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from hypolib import ld
from phaselib import read_chunk
from maskmodel import res as k3res
N0, D0 = ld(sys.argv[1]); mode = sys.argv[2]
ANCH = {'a': 0.8013, 'b': 0.9517, 'c': 1.1023, 'd': 1.2519, 'e': 1.3527, 'f': 1.4531,
        'g': 0.7311, 'h': 0.8741, 'i': 1.0309, 'j': 1.1789, 'k': 1.3021, 'l': 1.4129}
KK = [0] + list(range(int(os.environ.get('KMAX', 26)), 9, -1))
stages = [(c[0], int(c[1:])) for c in os.environ.get('STAGES', 'N7,N6,N5,N3,D6,D5,D4').split(',')]
if mode == 'control':
    if os.environ.get('TRUTHFILE'): Nt, Dt = ld(os.environ['TRUTHFILE'])
    else:
        from synth import make_truth
        Nt, Dt, mv, fc = make_truth(N0, D0, seed=int(os.environ.get('SEED', 1)), tsize=float(os.environ.get('TSIZE', 1e7)), kmin=15)
CASES = os.path.join(HERE, '..', '..', '..', '..', '..', 'eval', 'suites', 'probes-numeric', 'cases')
def run(tag, kk):
    a = ANCH[tag]; x0 = a if kk == 0 else a+2.0**-kk; xs = [x0]
    for _ in range(8191): xs.append(math.nextafter(xs[-1], math.inf))
    xs = np.array(xs)
    if mode == 'control': return xs, k3res(xs, Nt, Dt, kn=int(os.environ.get('KN', 99)))   # KN: T-2122 run 5
    o = os.path.join(CASES, 'kmyst-gam-k3l%s%02d.oracle.json' % (tag, kk))
    if not os.path.exists(o): return None
    v = json.load(open(o))['sheets']['Calc']; y = np.array([float(v.get('B%d' % (r+1), {}).get('v', np.nan)) for r in range(8192)])
    return xs, y
def stage_val(C, j, x):    # smooth (unrounded) S_j(x) = sum_{k>=j} C_k x^(k-j)
    return sum(float(C[k])*x**(k-j) for k in range(j, len(C)))
SL = []   # (chain, j, anchor x, slope, ulp(S_j))
for tag in os.environ.get('ANCH', 'abcdefghijkl'):
    reads = {}
    for kk in KK:
        r = run(tag, kk)
        if r is None: continue
        reads[kk] = read_chunk(N0, D0, r[0], r[1], stages)
    if 0 not in reads or len(reads) < 4: print('anchor %s: not enough runs (%d)' % (tag, len(reads))); continue
    print('anchor %s (x %.4f), %d runs' % (tag, ANCH[tag], len(reads)))
    for si, (ch, j) in enumerate(stages):
        b = reads[0][si]
        if abs(b[2]) < 0.12 or not (0.5 < b[1] < 2.0): print('   %s%d: base run unreadable' % (ch, j)); continue
        # quadratic unwrap: phase(dx) = s dx + s2 dx^2 / 2; predict each rung from the current (s, s2), take the
        # integer, then refit (s, s2) by LS over the unwrapped rungs (s2 only once >= 3 rungs)
        s, s2, used, DX, PH = 0.0, 0.0, [], [], []
        for kk in KK[1:]:
            if kk not in reads or kk < int(os.environ.get('KMIN', 10)): continue
            p = reads[kk][si]
            if abs(p[2]) < 0.12 or not (0.5 < p[1] < 2.0): continue
            dx = 2.0**-kk; meas = p[0]-b[0]; pred = s*dx+0.5*s2*dx*dx
            n = round(pred-meas); DX.append(dx); PH.append(meas+n); used.append(kk)
            Dx, Ph = np.array(DX), np.array(PH)
            if len(DX) >= 3 and os.environ.get('QUAD'):
                (s, s2), *_ = np.linalg.lstsq(np.c_[Dx, 0.5*Dx*Dx], Ph, rcond=None)
            else:
                s = float(np.dot(Dx, Ph)/np.dot(Dx, Dx))
        line = '   %s%d: slope %+12.1f cycles/x from %d rungs (%s)' % (ch, j, s, len(used), ','.join(str(k) for k in used))
        if len(used) >= int(os.environ.get('MINRUNG', 10)):
            SL.append((ch, j, ANCH[tag], s, math.ulp(abs(stage_val(N0 if ch == 'N' else D0, j, ANCH[tag]))), 2.0**max(used)))
        if mode == 'control':
            C, Ct = (N0, Nt) if ch == 'N' else (D0, Dt); a = ANCH[tag]
            u = math.ulp(abs(stage_val(C, j, a)))
            ts = sum((float(Ct[k])-float(C[k]))*(k-j)*a**(k-j-1) for k in range(j+1, len(C)))/u
            line += ' | truth %+12.1f' % ts
        print(line, flush=True)

# joint solve: slope_j(a) * ulp(S_j(a)) = sum_{k>j} dC_k (k-j) a^(k-j-1), unknowns dC_k for k > min readable j
for ch in 'ND':
    rows = [r for r in SL if r[0] == ch]
    if not rows: continue
    C = N0 if ch == 'N' else D0; kmin = min(r[1] for r in rows)+1; ks = list(range(kmin, len(C)))
    M = np.array([[(k-j)*a**(k-j-1) if k > j else 0.0 for k in ks] for _, j, a, _, _, _ in rows]); rhs = np.array([sl*u for _, _, _, sl, u, _ in rows])
    sc = np.abs(M).max(0); sc[sc == 0] = 1; uu = np.array([r[4] for r in rows]); keep = np.ones(len(rows), bool)
    per = np.array([r[5] for r in rows]); TOL = float(os.environ.get('TOL', 300)); rng = np.random.default_rng(0)
    def inl(sol):   # residual in cycles/x reduced modulo each equation's alias period
        rs = (M@sol-rhs)/uu; m = np.round(rs/per); return np.abs(rs-m*per) < TOL, m
    best = (-1, None)
    nk = len(ks)
    for _ in range(int(os.environ.get('RANSAC', 20000)) if len(rows) > nk else 1):   # consensus over minimal subsets
        sub = rng.choice(len(rows), min(nk, len(rows)), replace=False)
        try: sol = np.linalg.solve((M[sub]/sc), rhs[sub])/sc if len(sub) == nk else np.linalg.lstsq(M[sub]/sc, rhs[sub], rcond=None)[0]/sc
        except np.linalg.LinAlgError: continue
        k_, _ = inl(sol)
        if k_.sum() > best[0]: best = (k_.sum(), sol)
    keep, m = inl(best[1]); rhs = rhs+m*per*uu*keep
    sol, res_, rank, sv = np.linalg.lstsq(M[keep]/sc, rhs[keep], rcond=None); sol = sol/sc
    shifted = ['%s%d@%.4f:%+d' % (r[0], r[1], r[2], mm) for r, mm, kk_ in zip(rows, m, keep) if kk_ and mm != 0]
    print('   alias shifts: ' + (' '.join(shifted) or 'none'))
    print('%s chain: %d slope equations (%d kept), unknowns %s, rank %d' % (ch, len(rows), keep.sum(), ','.join('%s%d' % (ch, k) for k in ks), rank))
    print('   dropped: ' + (' '.join('%s%d@%.4f' % (r[0], r[1], r[2]) for r, k in zip(rows, keep) if not k) or 'none'))
    fitres = M@sol-rhs
    print('   residual per equation (in cycles/x): ' + ' '.join('%+.0f' % (f/u) for f, (_, _, _, _, u, _) in zip(fitres, rows)))
    line = '   solved moves (ulps): ' + ' '.join('%s%d %+.1f' % (ch, k, d/np.spacing(abs(C[k]))) for k, d in zip(ks, sol))
    if mode == 'control':
        Ct = Nt if ch == 'N' else Dt
        line += '\n   truth moves  (ulps): ' + ' '.join('%s%d %+.1f' % (ch, k, (Ct[k]-C[k])/np.spacing(abs(C[k]))) for k in ks)
    print(line)
