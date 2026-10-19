# W5 part 5: product-level phases on top of ss_fc (T-1941, K3)

Base: `../p4/ss_fc.txt` (plateau bits read, function corrected).
Synthetic controls use function-preserving moves: random combinations of the softest SVD directions of the
relative-function map, about 1e5 ulps, with function change of about 4e-17.

## Control-backed negatives
1. **Per-site phase meters fail when every stage differs** (`validate.py` with `inj.c`, `validate2.py` with
   `meter.py`).
   - The meters: injected-shift matched filter (discrete output flips), and a continuous template (the site's own
     rounding-error sequence at phase phi, times its propagation weight).
   - On a function-preserving synthetic move, per-run phases are random for every site (rms 0.2-0.47 cycles), at
     1,024, 4,096 and 8,192 points per run.
   - Cause (`dbg2.py`): a site's Excel error sequence is a phase-shifted copy of the candidate's only where its
     upstream product rounds the same way. For N.S7 the match is 65% at the true phase, because the upstream
     ulp(P7) = u(S7)/4 scrambles the rest.
   - The top sites (N.S7, N.S6, D.S6, D.S5) are also weak: template norm 0.3-2.7 against a residual norm of about
     45 per 8,192 points, so per-run SNR is about 1.
2. **No partial credit along the sloppy directions** (`lambda_path.py`). With truth = base + m, the set
   base + lambda·m scores exact 0.26-0.33 and residual correlation <= 0.17 for every lambda up to 0.9999 (about 4 ulps
   off on the mid coefficients). It scores 1.000 at lambda = 0.99999. The exact-match basin is about ±1-2 ulps on
   the mid coefficients. No whole-output objective (exact rate, correlation) guides a search there.
3. Bug found and fixed on the way: the residual is (B - ref)/|ref|, and res = t·A changes sign at x = 1, so every
   template or correction needs a sign(res) factor (`meter.py`, `../p4/fcorrect.py`).

## What does carry signal: coherent multi-run sums at the true phases (`coherent.py`)
Z of the sum over runs of the correlation at the *true* per-run phases, against random phase assignments, on the
synthetic:

| runs | N.S7 | N.P6 | N.P5 | D.S6 | D.P5 | D.P4 |
|---|---|---|---|---|---|---|
| 60 runs of 1,024 points | +3.0 | +1.5 | +5.1 | +2.3 | +3.7 | +6.3 |
| 43 runs of 4,096 points | +2.8 | +4.1 | +6.1 | +3.6 | +4.4 | +7.6 |

Taking each run's best phase gives Z of 8-14 at random, so per-run phases cannot be used. A global fit of each
site's smooth phase function, ΔS_j(x)/u, a polynomial with a few unknowns, is statistically feasible with all
real runs (about 700, so Z about 15-20 expected). Plan: a top-down global scan, N.S7 first, then one coefficient
per lower product site, FFT-accelerated.
