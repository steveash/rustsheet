"""K3 per-stage phase read (T-1941 W5 part 5; review 3's phase.py / phase_stages.py adapted to K3 and generalised).
Per run of consecutive doubles, the residual r = (y - exact)/ulp(y) carries each stored sum's rounding sawtooth as a
spectral line at f_j = S_j'(x) ulp(x) / ulp(S_j) cycles/step. The phase of Excel's line relative to the stored model's
(base set), signed by a planted +0.25-ulp offset at that sum, is the pre-rounding offset delta_j (ulp(S_j), mod 1).
Output per run and stage: x, phase, amplitude ratio |AE|/|AS|, calibration shift, line power / median.
usage: k3phase.py SET OUT.npy   env: MINLEN (4096), CH (8192 points per chunk), STAGES ("N8..N1,D8..D1")"""
import numpy as np, sys, os, math
from fractions import Fraction as F
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from hypolib import ld
from phaselib import read_chunk
N0, D0 = ld(sys.argv[1])
z = np.load(os.environ.get('NPZ', '/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows6.npz')); X, Y = z['x'], z['b']
br = np.where(np.nextafter(X[:-1], np.inf) != X[1:])[0]+1
MIN, CH = int(os.environ.get('MINLEN', 4096)), int(os.environ.get('CH', 8192))
chunks = [s[a:a+CH] for s in np.split(np.arange(len(X)), br) if len(s) >= MIN for a in range(0, len(s)-MIN+1, CH) if len(s[a:a+CH]) >= MIN]
chunks = [c for c in chunks if abs(X[c[0]]-1) > 1e-3]
stages = [(c[0], int(c[1:])) for c in os.environ.get('STAGES', 'N8,N7,N6,N5,N4,N3,N2,N1,D8,D7,D6,D5,D4,D3,D2,D1').split(',')]
out = []
for ci, c in enumerate(chunks):
    row = [float(X[c[len(c)//2]])]
    for v in read_chunk(N0, D0, X[c], Y[c], stages): row += list(v)
    out.append(row)
    if ci % 50 == 0: print('chunk %d/%d x %.4f' % (ci, len(chunks), row[0]), flush=True)
np.save(sys.argv[2], np.array(out)); print('saved', len(out), 'chunks x', len(stages), 'stages')
