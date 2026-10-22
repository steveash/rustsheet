"""K3 conditional (N6-first) solve on the 12-anchor KMAX=30 ladder reads (T-2122 run 2, HANDOFF2 13.4 step 1).
Reads the per-stage slopes printed by w5/p5/ladder.py (a log), then solves the N chain top-down instead of by a
joint RANSAC: N6's equations alone give (dN7, dN8, dN9); N7's reads are a consistency check; each lower stage then
adds one unknown (N5 -> dN6, N3 -> dN4 + dN5 jointly). Every equation is known only modulo its alias period
2^maxrung cycles/x, so each residual is reported reduced to that period and as the integer alias shift.
usage: k3_n6first.py LOG BASE   (BASE: ../w5/p4/ss_fc.txt)"""
import sys, re, math, os, itertools
import numpy as np
def ld(f):   # as w5/p5/hypolib.ld (without its compiled helpers)
    l = open(f).read().split('\n'); return [np.array([float.fromhex(v) for v in l[i].split()]) for i in (0, 1)]
N0, D0 = ld(sys.argv[2])
eq = []  # (chain, j, a, slope, per)
a = None
for line in open(sys.argv[1]):
    m = re.match(r'anchor (\w) \(x ([\d.]+)\)', line)
    if m: a = float(m.group(2)); continue
    m = re.match(r'\s+([ND])(\d): slope\s+([+-][\d.]+) cycles/x from (\d+) rungs \(([\d,]+)\)', line)
    if m and a is not None and int(m.group(4)) >= int(os.environ.get('MINRUNG', 10)):
        rungs = [int(r) for r in m.group(5).split(',')]
        eq.append((m.group(1), int(m.group(2)), a, float(m.group(3)), 2.0**max(rungs)))
def sval(C, j, x): return sum(float(C[k])*x**(k-j) for k in range(j, len(C)))
def row(C, j, x, ks): return np.array([(k-j)*x**(k-j-1) if k > j else 0.0 for k in ks])
def report(ch, j, ks, sol):
    C = N0 if ch == 'N' else D0
    out = []
    for c_, jj, x, s, per in eq:
        if c_ != ch or jj != j: continue
        u = math.ulp(abs(sval(C, j, x))); pred = row(C, j, x, ks)@sol/u
        r = s-pred; m = round(r/per); out.append((x, s, pred, m, r-m*per))
    return out
def fit(ch, j, ks, known, tol):
    """LS fit of the moves ks from stage j's equations, the moves in `known` (dict k -> dC) applied first; alias
    integers chosen by iterating LS -> nearest shift (start 0), reported per equation."""
    C = N0 if ch == 'N' else D0; rows = [(x, s, per) for c_, jj, x, s, per in eq if c_ == ch and jj == j]
    allk = sorted(set(ks) | set(known)); A = []; y = []; P = []; U = []
    for x, s, per in rows:
        u = math.ulp(abs(sval(C, j, x))); r = row(C, j, x, allk)
        kn = sum(r[allk.index(k)]*v for k, v in known.items())
        A.append([r[allk.index(k)] for k in ks]); y.append(s*u-kn); P.append(per); U.append(u)
    A, y, P, U = map(np.array, (A, y, P, U)); m = np.zeros(len(y))
    if len(y) < len(ks): return np.zeros(len(ks)), []   # stage not readable enough here
    # consensus start: every minimal subset (alias 0), the one with the most equations within tol
    best = (-1, None)
    for sub in itertools.combinations(range(len(y)), len(ks)):
        try: sol = np.linalg.solve(A[list(sub)], y[list(sub)])
        except np.linalg.LinAlgError: continue
        r = (A@sol-y)/U; n = (np.abs(r-np.round(r/P)*P) < tol).sum()
        if n > best[0]: best = (n, sol)
    r = (A@best[1]-y)/U; m = np.round(r/P); keep = np.abs(r-m*P) < tol
    for _ in range(20):
        sol = np.linalg.lstsq(A[keep]/U[keep, None], (y[keep]+m[keep]*P[keep]*U[keep])/U[keep], rcond=None)[0]
        r = (A@sol-y)/U; mn = np.round(r/P); res = r-mn*P
        nk = np.abs(res) < tol
        if (mn == m).all() and (nk == keep).all(): break
        m, keep = mn, nk
    return sol, [(x, mm, rr, kk) for (x, _, _), mm, rr, kk in zip(rows, m, res, keep)]
TOL = float(os.environ.get('TOL', 3e6))
def show(name, sol, ks, C, det):
    print('%s: moves %s' % (name, ' '.join('%s%d %+.4g ulps' % ('N', k, d/np.spacing(abs(C[k]))) for k, d in zip(ks, sol))))
    for x, mm, rr, kk in det: print('   x %.4f alias %+d residual %+12.0f cycles/x %s' % (x, mm, rr, '' if kk else 'DROPPED'))
# 1. N6 alone -> dN7, dN8, dN9 (quadratic in x)
s6, d6 = fit('N', 6, [7, 8, 9], {}, TOL); show('N6 eqs', s6, [7, 8, 9], N0, d6)
known = dict(zip([7, 8, 9], s6))
# 2. N7 check with (dN8, dN9) from N6
print('N7 check (predicted from N6 solve):')
for x, s, pred, mm, rr in report('N', 7, [8, 9], s6[1:]):
    print('   x %.4f read %+.4g pred %+.4g alias %+d residual %+.0f' % (x, s, pred, mm, rr))
# 3. N5 -> dN6
s5, d5 = fit('N', 5, [6], known, TOL); show('N5 eqs', s5, [6], N0, d5); known[6] = s5[0]
# 4. N3 -> dN4, dN5
s3, d3 = fit('N', 3, [4, 5], known, TOL); show('N3 eqs', s3, [4, 5], N0, d3); known[4], known[5] = s3
print('N moves (ulps): ' + ' '.join('N%d %+.4g' % (k, known[k]/np.spacing(abs(N0[k]))) for k in sorted(known)))
# 5. joint N6 + N7 (dN7, dN8, dN9): does one quadratic satisfy both stages? Weighted per equation by 1/ulp.
if os.environ.get('JOINT67'):
    rows = [(c_, jj, x, s, per) for c_, jj, x, s, per in eq if c_ == 'N' and jj in (6, 7) and not (jj == 6 and abs(x-0.9517) < 1e-4)]
    ks = [7, 8, 9]; A = []; y = []; P = []; U = []
    for _, jj, x, s, per in rows:
        u = math.ulp(abs(sval(N0, jj, x))); A.append(row(N0, jj, x, ks)); y.append(s*u); P.append(per); U.append(u)
    A, y, P, U = map(np.array, (A, y, P, U))
    best = None
    for shifts in itertools.product([-1, 0, 1, 2], repeat=sum(1 for r in rows if r[1] == 7)):
        m = np.zeros(len(y)); it = iter(shifts)
        for i, r in enumerate(rows):
            if r[1] == 7: m[i] = next(it)
        sol = np.linalg.lstsq(A/U[:, None], (y+m*P*U)/U, rcond=None)[0]
        res = (A@sol-y-m*P*U)/U; sc = np.sqrt(np.mean(res**2))
        if best is None or sc < best[0]: best = (sc, sol, m, res)
    sc, sol, m, res = best
    print('joint N6+N7: rms %.3g cycles/x, moves %s' % (sc, ' '.join('N%d %+.4g ulps' % (k, d/np.spacing(abs(N0[k]))) for k, d in zip(ks, sol))))
    for r, mm, rr in zip(rows, m, res): print('   N%d x %.4f alias %+d residual %+12.0f' % (r[1], r[2], mm, rr))
