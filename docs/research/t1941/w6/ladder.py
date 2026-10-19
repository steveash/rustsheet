# T-1941 W6: run-ladder readout for the lattice (q421-q433 layout, assumed until the generator is confirmed).
# Layout: per anchor a, an anchor run (n consecutive doubles from a) and rungs from a +- 2^-j (j = J0..J1), n doubles up.
# For each candidate law line: demodulate every run's R residual at the candidate's line frequency Fm (Hann window,
# phase referenced to the run centre), take the phase relative to the anchor run minus Fm*K, unwrap over the rungs
# (`ladder_unwrap.unwrap`, core by W4) and return F_meas = Fm + f: the line frequency at the anchor-run centre, mod 1,
# in cycles per double. Each (line, anchor) then gives one mod-1 congruence row for the lattice (`latsolve.LADDER`).
# Usage: ladder.py synth TRUTH CAND OUT [J0 J1 n]   (simulate the ladders from TRUTH and read them against CAND)
#        ladder.py excel RFILE CAND OUT              (read Excel's ladders; RFILE rows as rext.py writes them)
import sys, numpy as np

sys.path.insert(0, sys.path[0])
from synth import read_chain, simB
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


def layout(a, J0, J1, n):
    """[(K, x)] with K = (x0 - a)/ulp(a) the rung offset in doubles; K = 0 is the anchor run."""
    u = np.spacing(a)
    out = [(0, run_x(a, n))]
    for j in range(J1, J0 - 1, -1):  # small |K| first, as the unwrap needs
        for s in (1, -1):
            x0 = a + s * 2.0**-j
            out.append((int(round((x0 - a) / u)), run_x(x0, n)))
    return out


def lineF(c, law, j, xc):
    try:
        return stage_F(np.asarray(c), np.array([xc]))[key(law, j)][0][0]
    except KeyError:
        return None


def read(runs, N, D, lines, n):
    """runs: [(K, x, Rlo, Rhi)] of one anchor. Returns [(line, xc, F_meas, max|res|)]."""
    K0, x0, lo0, hi0 = runs[0]
    xc = x0[(n - 1) // 2]
    es = [(K, resid(x, lo, hi)) for K, x, lo, hi in runs]
    out = []
    for ch, law, j in lines:
        c = N if ch == "N" else D
        Fm = lineF(c, law, j, xc)
        if Fm is None:
            continue
        dK = 64.0
        g0 = (((lineF(c, law, j, xc + dK * np.spacing(xc)) - Fm + 0.5) % 1) - 0.5) / dK / 2
        ctr = np.exp(
            2j * np.pi * Fm * (n - 1) / 2
        )  # move the phase reference from sample 0 to the run centre
        z0 = demod(es[0][1], Fm) * ctr
        meas = []
        for K, e in es[1:]:
            z = demod(e, Fm) * ctr
            ph = np.angle(z / z0) / (2 * np.pi) - Fm * K
            meas.append((K, ((ph + 0.5) % 1) - 0.5))
        f, g, res = unwrap(meas, g0)
        out.append(((ch, law, j), xc, (Fm + f) % 1, float(np.max(np.abs(res)))))
    return out


def norm(p):
    N, D = read_chain(p)
    return list(np.array(N) / D[0]), list(np.array(D) / D[0])


if __name__ == "__main__":
    mode = sys.argv[1]
    Nc, Dc = norm(sys.argv[3])
    o = open(sys.argv[4], "w")
    if mode == "synth":
        Nt, Dt = norm(sys.argv[2])
        J0, J1, n = (int(v) for v in sys.argv[5:8]) if len(sys.argv) > 7 else (22, 46, 4096)
        for a in ANCHORS:
            runs = []
            for K, x in layout(a, J0, J1, n):
                B, _ = simB(Nt, Dt, x)
                lo, hi = extractR(x, B)
                runs.append((K, x, lo, hi))
            for ln, xc, F, mr in read(runs, Nc, Dc, LINES, n):
                Ft = lineF(Nt if ln[0] == "N" else Dt, ln[1], ln[2], xc)
                err = ((F - Ft + 0.5) % 1) - 0.5 if Ft is not None else float("nan")
                print(
                    "%.4f %s.%s%d F_meas %.15f truth-err %.2e max|res| %.3f" % (a, *ln, F, err, mr),
                    flush=True,
                )
                o.write("%.17g %s %s %d %.17g %.4f\n" % (xc, *ln, F, mr))
    else:
        raise SystemExit("excel mode: waiting for the confirmed q421-q433 layout")
