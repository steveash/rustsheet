"""Generate T-1778 round-3 ERFC-tail probe cases (label erfc-probes).

Writes kmyst-erfc-*.case.json into the probes-numeric suite. Cases (a)-(e) are the round-2 erfc-structure
proposals (reviewed; regenerated here with the same layout, see README), (f)-(h) are new.
"""
import json, math, os, sys
OUT = sys.argv[1] if len(sys.argv) > 1 else '/home/user/rustsheet/eval/suites/probes-numeric/cases'
TAGS = ['M9', 'probe', 'numeric', 'erf']
LAYOUT = {}   # case -> list of (run_id, first_row, n, label)
PHI = (math.sqrt(5) - 1) / 2

def write(name, desc, rows, cols, runs=None):
    cells = {}
    for i, x in enumerate(rows, start=1):
        cells[f'A{i}'] = x
        for col, f in cols:
            if callable(f):
                cells[f'{col}{i}'] = f(x)
            else:
                cells[f'{col}{i}'] = f.replace('#', str(i))
    assert len(cells) <= 70000, (name, len(cells))
    case = {'format': 'rustsheet-case/v1', 'description': desc, 'tags': TAGS, 'expected': {},
            'sheets': [{'name': 'Calc', 'cells': cells}]}
    with open(os.path.join(OUT, name + '.case.json'), 'w') as fh:
        json.dump(case, fh, indent=1)
    LAYOUT[name] = {'cols': [c for c, _ in cols], 'runs': runs or []}
    print(name, len(rows), 'rows', len(cells), 'cells')

def run(x0, n, below=0):
    xs = [x0]; v = x0
    for _ in range(below):
        v = math.nextafter(v, -math.inf); xs.insert(0, v)
    v = x0
    while len(xs) < n:
        v = math.nextafter(v, math.inf); xs.append(v)
    return xs

def full(t):
    """A full-mantissa double next to t (low mantissa bits not structured)."""
    return t * (1 + PHI * 1e-9)

def build(spec, n, below=0, base=0):
    rows, runs = [], []
    for k, (label, x0) in enumerate(spec):
        runs.append((base + k, len(rows) + 1, n, label, x0)); rows += run(x0, n, below)
    return rows, runs

def rdesc(runs):
    return '; '.join(f'run={r} {lab} x0={x0.hex()} n={n} row={row}' for r, row, n, lab, x0 in runs)

# (b) split at 0.5: 512 doubles below 0.5, 0.5, 511 above.
rows, runs = build([('x0=0.5 (row 513)', 0.5)], 1024, below=512, base=0)
runs = [(0, 1, 1024, 'split05: rows 1-512 below 0.5, row 513 = 0.5', rows[0])]
write('kmyst-erfc-split05',
      'Kernel-mystery probe (T-1778 round 3, erfc-probes): where exactly does ERF/ERFC switch from the small-x erf rational to the erfc kernel e, and is 0.5 itself on the small side? '
      'Below the switch ERFC(x) = 1 - ERF(x) exactly; above it ERF(x) = fl(1 - e) and ERFC(-x) = fl(2 - e), so ERFC != 1-ERF at about half the points just above the switch. '
      'Consecutive doubles: rows 1-512 are the 512 doubles below 0.5, row 513 is 0.5, rows 514-1024 the doubles above. A = x, B = ERF.PRECISE(x), C = ERFC.PRECISE(x), D = ERFC.PRECISE(-x). Runs: ' + rdesc(runs),
      rows, [('B', '=ERF.PRECISE(A#)'), ('C', '=ERFC.PRECISE(A#)'), ('D', '=ERFC.PRECISE(-A#)')], runs)

# (b') coarse localisation of the switch over the round-2 bracket (0.49922, 0.517114).
rows = [0.4990 + k * 1e-5 + PHI * 1e-7 for k in range(1850)]
runs = [(1, 1, len(rows), 'split05b: grid 0.4990 + k*1e-5 (+6.2e-8), k=0..1849', rows[0])]
write('kmyst-erfc-split05b',
      'Kernel-mystery probe (T-1778 round 3, erfc-probes): coarse localisation of the ERF/ERFC branch switch over the round-2 bracket (0.49922, 0.5171), in case it is not at 0.5 exactly. '
      'Rows are x = 0.4990 + k*1E-5 + 6.18E-8 for k = 0..1849 (not consecutive doubles). A = x, B = ERF.PRECISE(x), C = ERFC.PRECISE(x), D = ERFC.PRECISE(-x).',
      rows, [('B', '=ERF.PRECISE(A#)'), ('C', '=ERFC.PRECISE(A#)'), ('D', '=ERFC.PRECISE(-A#)')], runs)

# (a) decouple: consecutive z, CHISQ.DIST.RT(2z,1) vs ERFC(SQRT(z)).
rows, runs = build([(f'zc={zc}', zc) for zc in [0.3, 1.0, 4.0, 9.0, 30.0, 100.0, 300.0, 600.0]], 256, base=0)
write('kmyst-erfc-decouple',
      'Kernel-mystery probe (T-1778 round 3, erfc-probes): does the upper-tail path read z beyond fl(sqrt z)? Consecutive z doubles, so about half of neighbouring z share fl(sqrt(z)). '
      'If B = C everywhere the erfc kernel is entered with x = fl(sqrt z) and recomputes z = fl(x*x); if B differs between two z that share fl(sqrt z), the exp(-z) factor reads z directly and the x-rational is held fixed, which decouples the two factors. '
      'A = z, B = CHISQ.DIST.RT(2*z,1), C = ERFC.PRECISE(SQRT(z)), D = SQRT(z), E = GAMMA.DIST(z,0.5,1,TRUE). Runs of 256 (A = z): ' + rdesc(runs),
      rows, [('B', '=CHISQ.DIST.RT(2*A#,1)'), ('C', '=ERFC.PRECISE(SQRT(A#))'), ('D', '=SQRT(A#)'), ('E', '=GAMMA.DIST(A#,0.5,1,TRUE)')], runs)

# (d) expcol
rows, runs = build([(f'xc={xc}', xc) for xc in [0.75, 1.5, 3.0, 5.0, 7.9, 12.0, 20.0, 26.0]], 512, base=0)
write('kmyst-erfc-expcol',
      "Kernel-mystery probe (T-1778 round 3, erfc-probes): ERFC.PRECISE on runs of 512 consecutive x with Excel's own exp(-fl(x*x)) beside it, so the implied x-rational F = ERFC/EXP is known to half an ulp and the noise of the rational can be studied without assuming a libm exp. "
      'A = x, B = ERFC.PRECISE(x), C = EXP(-(x*x)), D = x*x. Runs: ' + rdesc(runs),
      rows, [('B', '=ERFC.PRECISE(A#)'), ('C', '=EXP(-(A#*A#))'), ('D', '=A#*A#')], runs)

# (c) branchscan
rows, runs = build([(f'b={b}', b) for b in [1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 5.0, 6.0, 7.0, 8.0, 10.0, 12.0, 16.0, 20.0]], 256, below=128, base=0)
write('kmyst-erfc-branchscan',
      'Kernel-mystery probe (T-1778 round 3, erfc-probes): is there a branch (a second rational, a term-count change) at candidate breakpoints b? Each run is the 128 doubles below b, b itself (the 129th row of the run) and the 127 doubles above. '
      'A jump in the mean of ERFC/(exp(-fl(x*x))*erfcx_true) across b larger than about 0.5 ulp (128-point means have SE 0.17 ulp) marks a branch; the step at b itself tells x<b from x<=b. '
      'A = x, B = ERFC.PRECISE(x), C = EXP(-(x*x)). Runs (x0 = first row): ' + rdesc(runs),
      rows, [('B', '=ERFC.PRECISE(A#)'), ('C', '=EXP(-(A#*A#))')], runs)

# (e) driftmap
n = 8000; lo, hi = 0.5, 26.54
step = (hi - lo) / n
rows = [lo + (k + 0.5 + 0.3819660112501051 * ((k * 7) % 5 - 2) / 5) * step for k in range(n)]
write('kmyst-erfc-driftmap',
      "Kernel-mystery probe (T-1778 round 3, erfc-probes): ERFC.PRECISE at 8,000 points in (0.5, 26.54), four times denser than kmyst-erfc-dense, with Excel's own exp(-fl(x*x)), to map the smooth +-2.5 ulp error curve of the erfc rational (its extrema, their spacing near x = 8 and 26.6, and any jumps) to about 0.2 ulp per 0.25-wide bin. "
      'Row k+1 (k = 0..7999) is x = 0.5 + (k + 0.5 + 0.381966*(((7k) mod 5) - 2)/5)*0.003255. A = x, B = ERFC.PRECISE(x), C = EXP(-(x*x)).',
      rows, [('B', '=ERFC.PRECISE(A#)'), ('C', '=EXP(-(A#*A#))')], [(0, 1, n, 'driftmap', rows[0])])

# (f1) ERFC runs ladder 0.505 .. 26.5, 60 runs, ratio 1.0694
R = 60; x_lo, x_hi = 0.505, 26.5; r = (x_hi / x_lo) ** (1 / (R - 1))
spec = [(f'xt={x_lo * r ** i:.6g} i={i}', full(x_lo * r ** i)) for i in range(R)]
for part in range(4):
    sub = spec[15 * part:15 * part + 15]
    rows, runs = build(sub, 1024, base=15 * part)
    write(f'kmyst-erfc-runs{part + 1}',
          f'Kernel-mystery probe (T-1778 round 3, erfc-probes): ERFC.PRECISE on a geometric x ladder from 0.505 to 26.5 (60 runs, ratio {r:.5f}; part {part + 1} of 4), each run 1,024 consecutive doubles upward from a full-mantissa x0 = xt*(1+6.18E-10), '
          "with Excel's own EXP(-(x*x)) beside it. A = x, B = ERFC.PRECISE(x), C = EXP(-(x*x)). Runs: " + rdesc(runs),
          rows, [('B', '=ERFC.PRECISE(A#)'), ('C', '=EXP(-(A#*A#))')], runs)

# (f2) upper-tail gamma path on consecutive z: CHISQ.DIST.RT(2z,1), ERFC(SQRT(z)), EXP(-z), SQRT(z)
R = 30; z_lo, z_hi = 0.26, 702.0; r = (z_hi / z_lo) ** (1 / (R - 1))
spec = [(f'zt={z_lo * r ** i:.6g} i={i}', full(z_lo * r ** i)) for i in range(R)]
for part in range(3):
    sub = spec[10 * part:10 * part + 10]
    rows, runs = build(sub, 1024, base=100 + 10 * part)
    write(f'kmyst-erfc-chiruns{part + 1}',
          f'Kernel-mystery probe (T-1778 round 3, erfc-probes): the upper-tail incomplete-gamma path on a geometric z ladder from 0.26 to 702 (30 runs, ratio {r:.5f}; part {part + 1} of 3), each run 1,024 consecutive doubles z upward from a full-mantissa z0. '
          'Feeds the erfc kernel a z that is not fl(x*x) of any stored x: comparing B with C (stored x = SQRT(z)) and with D = EXP(-z) separates the exp factor from the rational in x. '
          'A = z, B = CHISQ.DIST.RT(2*z,1), C = ERFC.PRECISE(SQRT(z)), D = EXP(-z), E = SQRT(z). Runs (A = z): ' + rdesc(runs),
          rows, [('B', '=CHISQ.DIST.RT(2*A#,1)'), ('C', '=ERFC.PRECISE(SQRT(A#))'), ('D', '=EXP(-A#)'), ('E', '=SQRT(A#)')], runs)

# (g) zero cutoff
rows = [26.5 + k * 0.002 for k in range(76)] + [26.5 + k * 0.0001 + PHI * 1e-6 for k in range(1001)]
write('kmyst-erfc-cutoff',
      'Kernel-mystery probe (T-1778 round 3, erfc-probes): where does ERFC.PRECISE(x) become 0 (subnormal flush of the result, or an x / z threshold such as sqrt(MAXLOG) = 26.64)? '
      'Rows 1-76: x = 26.50 + 0.002k (k = 0..75); rows 77-1077: x = 26.50 + 0.0001k + 6.18E-7 (k = 0..1000). '
      'A = x, B = ERFC.PRECISE(x), C = EXP(-(x*x)), D = CHISQ.DIST.RT(2*x*x,1), E = NORM.S.DIST(-x*SQRT(2),TRUE), F = x*x.',
      rows, [('B', '=ERFC.PRECISE(A#)'), ('C', '=EXP(-(A#*A#))'), ('D', '=CHISQ.DIST.RT(2*A#*A#,1)'), ('E', '=NORM.S.DIST(-A#*SQRT(2),TRUE)'), ('F', '=A#*A#')],
      [(0, 1, 76, 'coarse 26.50+0.002k', 26.5), (1, 77, 1001, 'fine 26.50+0.0001k+6.18e-7', rows[76])])

# (h) NORM.S.DIST(-y) on consecutive y vs ERFC(y*sqrt(0.5))/2 with constant arguments
spec = [(f'xe={xe} y=xe*sqrt2', full(xe * math.sqrt(2))) for xe in [0.6, 1.2, 3.0, 6.0, 12.0, 18.8]]
rows, runs = build(spec, 512, base=0)
s05 = math.sqrt(0.5); s2 = math.sqrt(2.0)
write('kmyst-erfc-norm',
      'Kernel-mystery probe (T-1778 round 3, erfc-probes): does NORM.S.DIST(-y,TRUE) equal ERFC.PRECISE(y/sqrt2)/2 bit for bit in the erfc tail, and which rounding of y/sqrt2 does it use? Runs of 512 consecutive y. '
      'A = y; B = NORM.S.DIST(-y,TRUE); C = y*sqrt(0.5) and D = y/sqrt(2) as constants (single IEEE double roundings, computed outside Excel); E = ERFC.PRECISE(C)/2; F = ERFC.PRECISE(D)/2; G = NORM.DIST(-y,0,1,TRUE); H = ERFC.PRECISE(A*SQRT(0.5))/2 (Excel arithmetic). Halving is exact. Runs: ' + rdesc(runs),
      rows, [('B', '=NORM.S.DIST(-A#,TRUE)'), ('C', lambda y: y * s05), ('D', lambda y: y / s2), ('E', '=ERFC.PRECISE(C#)/2'), ('F', '=ERFC.PRECISE(D#)/2'),
             ('G', '=NORM.DIST(-A#,0,1,TRUE)'), ('H', '=ERFC.PRECISE(A#*SQRT(0.5))/2')], runs)

with open(os.path.join(os.path.dirname(os.path.abspath(__file__)), 'layout.json'), 'w') as fh:
    json.dump(LAYOUT, fh, indent=1, default=float)
