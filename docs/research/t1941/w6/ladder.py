# T-1941 W6: run-ladder readout for the lattice (q421-q433 layout, assumed until the generator is confirmed).
# Layout: see SETS (confirmed by the coordinator: W4 gen_qrun.py and W6 gen_qrun_nd.py).
# For each candidate law line: demodulate every run's R residual at the candidate's line frequency Fm (Hann window,
# phase referenced to the run centre), take the phase relative to the anchor run minus Fm*K, unwrap over the rungs
# (`ladder_unwrap.unwrap`, core by W4) and return F_meas = Fm + f: the line frequency at the anchor-run centre, mod 1,
# in cycles per double. Each (line, anchor) then gives one mod-1 congruence row for the lattice (`latsolve.LADDER`).
# Usage: ladder.py synth dyadic|nd TRUTH CAND OUT   (simulate the ladder set from TRUTH and read it against CAND)
#        ladder.py excel dyadic|nd RFILE CAND OUT   (read Excel's ladders; RFILE rows as rext.py writes them)
import sys, numpy as np

sys.path.insert(0, sys.path[0])
from synth import read_chain, simB, nrm
from longspec import extractR, resid
from stagelaws import stage_F
from ladder_unwrap import unwrap, demod

ANCHORS = [1.625, 1.75, 1.875]
LINES = [(ch, "T", j) for ch in "ND" for j in range(2, 8)] + [(ch, "Pn1", j) for ch in "ND" for j in range(0, 8)]


def key(law, j):
    return ("T%d" % j) if law == "T" else "P%dn1" % j


def run_x(x0, n):
    x = np.empty(n)
    x[0] = x0
    for i in range(1, n):
        x[i] = np.nextafter(x[i - 1], np.inf)
    return x


# Ladder sets as generated (W4 gen_qrun.py, W6 gen_qrun_nd.py): rungs j = 22..46 step 2, starts fl(a + s*c*2^-j),
# NR consecutive doubles upward; the non-dyadic set also has K = 0 anchor runs (q440a).
SETS = {
    "dyadic": dict(anchors=[1.625, 1.75, 1.875], c=1.0, js=range(22, 47, 2), k0=False, cases=range(421, 434)),
    "nd": dict(anchors=[1.6341, 1.7718, 1.9253], c=1.37, js=range(22, 47, 2), k0=True, cases=range(440, 454)),
    # hypothetical top-weighted set near 2- (synthetic only: would top-stage lines become readable?)
    "top": dict(anchors=[1.962, 1.9745, 1.9871], c=1.37, js=range(22, 47, 2), k0=True, cases=()),
    # K2 (run with KERNEL=k2): q220a = K = 0 runs, q221..q233 = rungs; c = 1.37 assumed (W6 non-dyadic layout)
    "k2": dict(anchors=[2.30, 2.55, 2.80, 2.95], c=1.37, js=range(22, 47, 2), k0=True, cases=range(220, 234)),
}


def starts(a, c, js, k0):
    """[(K, x0)]: K = (x0 - a)/ulp(a), exact (both in the same binade)."""
    u = np.spacing(a)
    out = [(0, a)] if k0 else []
    for j in js:
        for s in (1, -1):
            x0 = a + s * c * 2.0**-j
            out.append((int(round((x0 - a) / u)), x0))
    return out


def lineF(c, law, j, xc):
    try:
        return stage_F(np.asarray(c), np.array([xc]))[key(law, j)][0][0]
    except KeyError:
        return None


def read(runs, N, D, lines, n):
    """runs: [(K, x, Rlo, Rhi)] of one anchor. Reference run: K = 0 if present, else the smallest positive K; phases are
    referenced to each run's centre and taken relative to the reference run, so F_meas is the line frequency at the
    reference-run centre xc. Returns [(line, xc, F_meas, max|res|)]."""
    ks = [r[0] for r in runs]
    ref = runs[ks.index(0)] if 0 in ks else min((r for r in runs if r[0] > 0), key=lambda r: r[0])
    Kr, xr = ref[0], ref[1]
    xc = xr[(n - 1) // 2]
    es = [(K - Kr, resid(x, lo, hi)) for K, x, lo, hi in runs if K != Kr]
    es.sort(key=lambda t: abs(t[0]))  # small |K| first, as the unwrap needs
    er = resid(xr, ref[2], ref[3])
    out = []
    for ch, law, j in lines:
        c = N if ch == "N" else D
        Fm = lineF(c, law, j, xc)
        if Fm is None:
            continue
        dK = 64.0
        g0 = (((lineF(c, law, j, xc + dK * np.spacing(xc)) - Fm + 0.5) % 1) - 0.5) / dK / 2
        ctr = np.exp(2j * np.pi * Fm * (n - 1) / 2)  # move the phase reference from sample 0 to the run centre
        z0 = demod(er, Fm) * ctr
        meas = []
        for K, e in es:
            z = demod(e, Fm) * ctr
            ph = np.angle(z / z0) / (2 * np.pi) - (Fm * K) % 1
            meas.append((K, ((ph + 0.5) % 1) - 0.5))
        f, g, res = unwrap(meas, g0)
        out.append(((ch, law, j), xc, (Fm + f) % 1, float(np.max(np.abs(res)))))
    return out


def norm(p):
    N, D = read_chain(p)
    return list(np.array(N) /nrm(D)), list(np.array(D) /nrm(D))


def excel_runs(rfile, st, n):
    """Map Excel rows ("x B Rlo Rhi ncand case", rext.py) of the set's cases onto the expected runs, by exact start."""
    pref = tuple("kmyst-gam-q%d" % k for k in st["cases"])
    pts = {}
    for l in open(rfile):
        q = l.split()
        if q[5].startswith(pref):
            pts[float.fromhex(q[0])] = (float.fromhex(q[2]), float.fromhex(q[3]))
    out = {}
    for a in st["anchors"]:
        runs = []
        for K, x0 in starts(a, st["c"], st["js"], st["k0"]):
            x = run_x(x0, n)
            if all(v in pts for v in x):
                runs.append((K, x, np.array([pts[v][0] for v in x]), np.array([pts[v][1] for v in x])))
            else:
                print("missing run a=%g K=%d (%d/%d points)" % (a, K, sum(v in pts for v in x), n), flush=True)
        out[a] = runs
    return out


if __name__ == "__main__":
    mode, setname = sys.argv[1], sys.argv[2]
    st = SETS[setname]
    n = 4096
    Nc, Dc = norm(sys.argv[4])
    o = open(sys.argv[5], "w")
    if mode == "synth":
        Nt, Dt = norm(sys.argv[3])
        allruns = {}
        for a in st["anchors"]:
            runs = []
            for K, x0 in starts(a, st["c"], st["js"], st["k0"]):
                x = run_x(x0, n)
                B, _ = simB(Nt, Dt, x)
                lo, hi = extractR(x, B)
                runs.append((K, x, lo, hi))
            allruns[a] = runs
    else:
        allruns = excel_runs(sys.argv[3], st, n)
    for a, runs in allruns.items():
        if len(runs) < 4:
            continue
        for ln, xc, F, mr in read(runs, Nc, Dc, LINES, n):
            msg = "%.4f %s.%s%d F_meas %.15f" % (a, *ln, F)
            if mode == "synth":
                Ft = lineF(Nt if ln[0] == "N" else Dt, ln[1], ln[2], xc)
                msg += " truth-err %.2e" % (((F - Ft + 0.5) % 1) - 0.5 if Ft is not None else float("nan"))
            print(msg + " max|res| %.3f" % mr, flush=True)
            o.write("%.17g %s %s %d %.17g %.4f\n" % (xc, *ln, F, mr))
