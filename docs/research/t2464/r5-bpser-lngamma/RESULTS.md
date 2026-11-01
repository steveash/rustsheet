# T-3570 r5-bpser-lngamma: lnGamma(a0 < 1) inside bpser's L

Round 5, 2026-11-01. Question (r2-bpser-small RESULTS.md section 3 item 8, section 4): inside bpser's
L = fl(g(a0) + fl(GL(b0) - GL(a0 + b0))), is g(a0) for a0 < 1 Excel's GAMMALN (GL53) or TOMS's
fl(gamln1(a0) - LN(a0)) (L53), and does the answer depend on the shape?

- Python: `bindings/python/.venv/bin/python` (repo-relative; `make py-bindings`, plus `uv pip install mpmath numpy
  scipy` into it), run from this directory. The scripts import r2-bpser-small's `lib.py` (and through it r2-bratio's
  `bratio_excel.py`); nothing is copied.
- Scores are bit-exact / within 4 ulp / within 15 significant digits.

## 1. Probe

`make_probe.py` wrote two probes-numeric cases, filled from excel-graph on 2026-11-01 with 0 failures:

- **t3570-r5-bpser-lngamma-beta** (1,920 cells): `BETA.DIST(x, a0, b0, TRUE)` at b0 = 2.5 and 6.25 (TOMS sub-branch
  1 < b0 < 8, direct orientation, bratio's `w_bpser` for every cell). a0 steps through 160 consecutive doubles around
  0.3, 0.5 and 0.7, at two short-decimal x per b0 (2.5: 0.0371, 0.0913; 6.25: 0.0213, 0.0371). Exact doubles are
  written as w2-forensics' `num` expressions, so no literal is cut to 15 digits.
- **t3570-r5-bpser-lngamma-gl** (620 cells): `GAMMALN` at every a0 and fl(a0 + b0) of the beta probe. Every L below
  uses these measured values, never the engine's GAMMALN model (r2-bpser-small item 2: that model is exact on only
  1,738 of 2,585 arguments below 1).

Since b0 and x are fixed along each run, only g(a0) and GL(a0 + b0) move from cell to cell. Tune = b0 2.5, hold-out =
b0 6.25, fixed before scoring.

## 2. Result: g(a0) is Excel's GAMMALN at 0.3, 0.5 and 0.7 (`score_forms.out`)

| set | cells | GL53 | L53 | cells where the forms differ: GL53 right / L53 right / neither |
|---|---|---|---|---|
| tune, b0 2.5 | 960 | **672**/960/960 | 573/960/960 | 216: **149** / 50 / 17 |
| hold-out, b0 6.25 | 960 | **831**/960/960 | 691/960/960 | 241: **182** / 42 / 17 |

GL53 wins in every one of the 12 runs (b0, center, x) and on both sides of every center. At (6.25, 0.7) it is
bit-exact on all 320 cells, against 268 for L53. Around 0.5 and 0.7 this contradicts r2-bpser-small's reading
(gbins_mid.out: "GL loses or ties above 0.7"; the old w2f (0.5, 6.25) shapes "prefer g1ln53"). Those readings came
from shape sets where the shape, x and both lnGamma pieces all varied at once. Here only a0 moves, so the comparison is
direct. **lnGamma(a0 < 1) inside L is not shape-dependent on this evidence: it is GL(a0).** The engine's bpser already
uses GL53's L (`functions/statistical/bratio/bpser.rs`, T-2505), so no engine change follows. Its remaining gap is
GAMMALN's own accuracy below 1, which T-3042 owns.

Scope: only a0 within 80 ulps of 0.3, 0.5 and 0.7, at b0 2.5 and 6.25, in the direct orientation. Small a0, b0 <= 1 and
b0 >= 8 are not covered. Two things are not checked here and go to T-3769. One is whether r2-bpser-small's old w2f
(0.5, 6.25) reading changes when it is re-scored with measured GAMMALN. The other is whether these 1-2 ulp bits
reproduce on a second excel-graph fill (decision 0112: bits that vary between fills are not targets).

The method needs no fitted constant. It compares two fixed formulas cell by cell, so there is no `FITTED` value and no
R4 fit to hold out. The tune/hold split by b0 still guards against a b0-specific accident. 457 of the 1,920 cells tell the two forms apart (a discrimination check, not a known-kernel positive control), and
the winner is the same on the tune and hold-out halves.

## 3. What GL53 still misses (`residual.out`, `lshift.out`)

GL53 misses 417 / 1,920 bit-exact. Every miss is within 2 ulp, so within 15 digits.

- **At 0.3 the misses follow x, not a0.** At (2.5, 0.3), 52 and 50 misses at the two x share 14 a0, against 16.2
  expected if they were independent. At (6.25, 0.3) all 44 misses are at x = 0.0371 and none at 0.0213. Every one is
  the model 1 ulp above Excel. This is the x-dependent single-intermediate pattern of r2-bpser-small section 4 (the
  b < 1 ceiling, there for b < 1), seen here at 1 < b0 < 8. It is not L.
- **At 0.5 and 0.7 part of the residual follows a0.** Both x miss together at 19 vs 10.7 expected (2.5, 0.5), 32 vs
  16.5 (2.5, 0.7) and 28 vs 11.0 (6.25, 0.5). Offsets are +1 or +2 ulp (model above), with a few -1 and -2.
- **No constant L shift explains it.** `lshift.py` moves GL53's L by k = -2..2 ulps and asks which k makes both x of an
  a0 exact. Of the 960 a0: 636 fit at k = 0, 155 only at some k != 0, mostly +1 or +2 (so Excel's L is larger there),
  and 169 at no k in -2..2. The k that fits changes from one a0 to the next, so this is not one rounding of L. Two
  candidates remain, both untested here: Excel computes lnGamma(a0) internally at a different precision than the
  GAMMALN worksheet function returns, or alx = fl(a LN x) rounds differently.

## 4. Not done in this round

- **The b < 1 bpser ceiling** (r2-bpser-small section 4: one stored intermediate, s or E, beating with x). Not
  attempted. Section 3's x-linked misses at a0 near 0.3 look like the same effect in 1 < b0 < 8, so a probe there
  can study it without b < 1. Follow-up: T-3769.
- The a0-linked residual at 0.5 and 0.7 (section 3) goes to the same follow-up.

## 5. Files

- `make_probe.py`: writes the two cases and their `*.meta.json` (cell → (x, a0, b0)).
- `score_forms.py` → `score_forms.out`: GL53 against L53 per run, side of center and tune/hold-out.
- `residual.py` → `residual.out`: whether GL53's misses co-occur across the two x of an a0.
- `lshift.py` → `lshift.out`: which constant L shift, if any, fits both x of each a0.
