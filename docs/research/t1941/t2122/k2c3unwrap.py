# T-2122 K2: unwrap the per-run N.s3 offset c3(x) (HANDOFF2 section 12.1) along the micro-ladders (gen_k2micro.py).
# Reads SCRATCH/c3all.pkl from review3/final/c3all.py (run it after the k2m oracles land). Per ladder (anchor a..e and
# near-2 z), runs sorted by x; consecutive differences of c3 taken mod 1 into [-0.5, 0.5) and summed. Coherence of
# the steps says whether 2e-5 spacing resolves the law (random steps: coherence ~0, mean |dc| 0.25).
# Then fits the unwrapped c3 against x with polynomial degree 1..3 and reports the slope in cycles per unit x.
# usage (repo root): python3 docs/research/t1941/t2122/k2c3unwrap.py [MINGAIN]
import pickle, sys, numpy as np

rows = pickle.load(open("SCRATCH/c3all.pkl", "rb"))
MING = float(sys.argv[1]) if len(sys.argv) > 1 else 0.0
lad = {}
for x, n, c3, s0, s1, case in rows:
    if not case.startswith("k2m"):
        continue
    tag = case[3]
    if s1 - s0 < MING:
        continue
    lad.setdefault(tag, []).append((x, c3, s0, s1))
for tag in sorted(lad):
    R = np.array(sorted(lad[tag]))
    x, c, s0, s1 = R.T
    d = (np.diff(c) + 0.5) % 1 - 0.5
    coh = abs(np.mean(np.exp(2j * np.pi * d))) if len(d) else 0
    u = np.concatenate([[c[0]], c[0] + np.cumsum(d)])
    print(
        f"ladder {tag}: {len(x)} runs, x {x[0]:.6f}..{x[-1]:.6f}, mean score {s0.mean():.4f} -> {s1.mean():.4f}, "
        f"step mean|dc| {np.mean(abs(d)):.3f} coherence {coh:.3f}"
    )
    for deg in (1, 2, 3):
        if len(x) > deg + 2:
            p = np.polyfit(x - x[0], u, deg)
            res = u - np.polyval(p, x - x[0])
            print(
                f"   deg {deg}: slope at start {np.polyval(np.polyder(p), 0):+.1f} cycles/x, rms {np.sqrt(np.mean(res**2)):.3f}"
            )
    step = max(1, len(x) // 15)
    print(
        "   x, c3, unwrapped: "
        + " ".join(f"{xi:.5f}:{ci:+.3f}/{ui:+.2f}" for xi, ci, ui in zip(x[::step], c[::step], u[::step]))
    )
