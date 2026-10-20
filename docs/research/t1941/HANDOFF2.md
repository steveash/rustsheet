# T-1941 handoff v2: Excel GAMMALN, GAMMA and LN kernels (state at 2026-10-20 08:00 UTC)

This file is the **restart point**. It is self-contained: read it top to bottom before touching anything. The long
chronological log, with every experiment and number, is `HANDOFF.md` (sections 1-12.20) and `README.md` in this
directory. Where they disagree with this file, this file wins.

- **Branch:** `agent/research/T-1941-crack-gammaln-gamma-ln` (research only; never force-push; commits are
  `T-1941: ...` with the Co-Authored-By / Claude-Session trailers).
- **Task:** `docs/tasks/M9/T-1941-crack-excel-s-gammaln-gamma-and-ln-kernels.md`; continuation task: see section 10.
- **Data:** `eval/suites/probes-numeric/cases/kmyst-gam-*` (about 1,100 cases, ~20 M rows; oracle values from
  Excel via `tools/` `rustsheet-oracle fill -b excel-graph`). Extract with `extract_runs.py` (selects columns by
  formula; never by letter). Oracle data are noise-free: 551,309 duplicated x across cases, zero conflicts.

## 1. The bar and the arithmetic model

- Match Microsoft 365 Excel bit for bit. Excel evaluates on the x87 FPU: every stored op is fl53(fl64(.))
  (64-bit register rounding, then a store to double). Pure SSE2 double arithmetic is measurably wrong (K2 near-2
  ladders: 0.9932 vs 0.9953 for x87 double rounding; K2 products: 0.937 vs 0.952).
- Transcendentals use the x87 instructions: LN = FLDLN2 + FYL2X, EXP = model V6 (T-1778), SIN = FSIN.
- The four open "kernels" are private rational approximations N/D in raw x (or y), evaluated by Horner, with
  Excel's own coefficient doubles. They are not any published table (Cody, Cephes, Boost, Hart, NR, fdlibm all
  checked and wrong to the bit).

## 2. Solved pieces (bit-exact or within x87 near-ties; verified on committed data)

| Piece | Model | Score |
|---|---|---|
| LN(x) | RN53(RN64(RN64(ln2)·log2 x)) (FLDLN2 + FYL2X) | 114,687 / 114,688 (1 FYL2X near-tie) |
| GAMMALN, x ≥ 8 | 10-term Bernoulli Stirling, every op stored (`gammaln_big_check.c`) | 291,957 / 291,957 |
| GAMMALN, x < 0.7 (exact cut 0x1.6666666666666p-1) | fl(GAMMALN(fl(x+1)) − LN(x)) | 12,288 / 12,288 given the kernel |
| GAMMALN, 3 ≤ x < 8 | fl(Gbase(t) + LN(p)), t in [2,3), p the ascending stored product | 100% given Excel's columns |
| GAMMALN legacy | = GAMMALN.PRECISE | identical |
| GAMMA, x ≥ 10 | binary-powering x^(x/2) with EXP V6 / LN (README) | 155,714 / 155,964 (99.84%) |
| GAMMA, 1 ≤ x < 10 | descending stored product × G(y), y in [1,2) | 52,224 / 52,224 given G |
| GAMMA, 0 < x < 1 | fl(fl(1/x)·G(fl(x+1))) | 18,431 / 18,432 |
| GAMMA, −10 < x < 0 | z = 1/x; y = x+1; while y < 1 { z /= y; y += 1 }; z·G(y) | 18,176 / 18,176 given G |
| GAMMA, x ≤ −10 | ±fl(fl(π/GAMMA(q))/fl(q·s)), s = |RN53(FSIN(RN53(π·z)))| | 6,143 / 6,144 (1 FSIN tie) |
| GAMMA half-integers | ascending product × √π double; negatives via π/(Gh·q) | 172/172, 171/171 |
| GAMMA integers ≤ 88 | downward product | exact |
| GAMMALN tiny x | fl(−LN(x) + fl(x·(−γ))) | 16,384 / 16,384 (≈ x<0.7 path) |

Everything else reduces to four open kernels:

- **K1:** GAMMA's G(y), y in [1,2) (used by all of GAMMA below 10).
- **K2:** GAMMALN on (2,3] (also the Gbase of GAMMALN on [3,8)).
- **K3:** GAMMALN near 1, [0.7, 1.5) (also reached by every x < 0.7 through the recurrence).
- **(1.5,2]:** GAMMALN on (1.5, 2] (different composition: fl((x−1)·fl((x−2)·R)), R a rational).

## 3. Open kernels: best models now (all plain doubles, stored x87 Horner unless noted)

| Kernel | Best model file | Exact | Within 1 ulp | Within 2 ulp | Max |
|---|---|---|---|---|---|
| K2 (2,3] | `k2_ref.py` (coefficients `k2_plain.txt` family; see 3.2 on structure) | 0.950 (14 runs), 0.954 (1,024 clusters) | 0.978 | 0.994 | 7 ulp |
| K1 [1,2) | `w4/k1r/plain1.txt` | 0.776 (3.59 M unique pts) | 0.878 | 0.959 | 7 ulp |
| K1 by region | [1,1.4) 0.877 / 0.956; [1.4,1.8) 0.856 / 0.917; [1.8,2) 0.504 / 0.705 | | | | |
| K3 [0.7,1.5) | `w5/p4/ss_fc.txt` | 0.33 on rows6 (5.68 M pts); 0.29 on new k3t | ~0.75 (est.) | | |
| (1.5,2] | `w6/data/xl_chain_pcD0.txt` + D3 −2,070,784 ulps + low-block re-solve (W6) | 42.7% exact on R | | | |

For comparison, a correctly rounded lgamma (what the engine has today for GAMMALN, via fdlibm) matches Excel at
0.29-0.35 exact, 0.74 within 1 ulp, 0.92 within 2 ulp, max 5 ulp on K2 data. Our models are far better at the
bit level but the residual misses are a few ulp (1e-16 relative), never visible at 15 significant digits.

### 3.1 How K2 was taken from 0.74 to 0.95 today (the recipe; it transfers)

1. **Model-free phase read** (`review3/phase.py`): per dense cluster (512 consecutive doubles), the phase of the
   hand-off stage's sawtooth line in (Excel − exact) relative to (sim − exact), calibrated with a planted
   +0.25-ulp offset, reads that stage's pre-rounding offset mod 1 ulp. Across 1,024 clusters the offsets followed
   a smooth law δ(x) = A + B/x (N: B = −3131 u, D: +553 u, u = 2^-54). It looked "erratic" earlier only because the
   slope is ~600 ulps per unit x and clusters were 0.001-0.003 apart.
2. **The law is a coefficient move, not an op:** a hidden "+c/x at the hand-off product, −c at the next
   constant" model (algebraically a no-op) reproduced it (`review3/cx.py`, 0.88-0.91), and W2 then showed it is
   a proxy for **plain different head coefficients**: head += c·g(x) with g a CUBIC ≈ 1/x on [2,3] (cubic minimax
   alone: 0.905; quadratic fails: 0.45), next coefficient −= c, late chain unchanged. Only p1..p5 / q0..q4 differ
   from grid_a, by up to 1.5e6 ulps along a function-near-null direction.
3. **Null-lattice refinement** (fpylll LLL basis of near-null integer moves; greedy multi-scale; planted control
   first) to 0.938 stored / 0.952 with the head mask; the late coefficients are only identifiable up to plateaus.
4. Why every earlier search missed it: single-coefficient needles, LS re-solves that moved the LATE chain, and
   per-run offset sweeps whose integer part leaked into the output could not represent "head moves along c·g,
   next −c, late fixed".

### 3.2 K2 structure: open question (stored vs 80-bit head)

- With the converged coefficients, keeping N's s1..s3 and D's d1..d2 in 80-bit (stored from the hand-off product
  on) scores 0.952 vs 0.938 all-stored on [2,3]; `k2_ref.py` carries that mask (with a CAUTION note).
- But the **near-2 ladders** (e4/e5/r204 families, 139k points, x−2 < 5e-7), where grid_a scores 0.9953
  stored, favour all-stored (one extended head sum costs 3 points there).
- Near-2 linear constraint (s5(2) unchanged): 16δp1 + 8δp2 + 4δp3 + 2δp4 + δp5 ≈ 0 to ≲0.05u; D side
  Σ δq_k 2^{4−k} ≈ 0.
- The decisive control (reviewer 3, running at handoff): refit on FAR data only under each structure, score on
  the near-2 ladders. Whichever transfers to ≥ 0.995 near 2 is the real structure.
- Residual after the best model (~5%): distributed, 50% of misses ≥ 2 ulp, mild boundary enrichment at the
  hand-off product/sum (≤ 2×), a smooth multi-extremum cluster-mean bias (cubic-minimax-like signs, worst at
  x = 2.0-2.05). Not a per-run phase (per-run caps 0.959), not coefficients (all refits flat), not 80-bit
  constants, not head scheme, not quotient/final ops, not low bits of x.

### 3.3 K1 state

- Structure: raw-y stored Horner (7,7), G = fl(N/D), D0 = −1/16 (a power of two; normalisation is safe).
- Two hidden no-op shifts (N at c2: +852·ulp(N1); D at c3: −350·ulp(D2)) were found by scans and absorbed into
  plain coefficients via e·g(y), g ≈ 1/y (degree 5/4 needed; cubic fails), then an integer climb → plain1.
- [1, 1.8): per-run caps 0.93-0.97, so smooth site laws remain (reachable).
- [1.8, 2) "dip" (worst at y ≈ 1.888-1.90): W4's per-run caps (0.65 sums / 0.80 all 26 sites / 0.64 no-op)
  were shown by a planted control to be LOCAL under-estimates (a pure coefficient change also caps at ~0.85), and
  W5's regional fingerprint sees no extra noise there. Verdict: a coefficient problem; D S2's cancellation
  magnifies the upper D coefficients there, so the true set is far from plain1 along directions that matter in
  the dip. A dip-only set (≤ 4 ulp from plain1) lifts the dip to 0.57 and destroys the rest.
- Rejected for the dip: exp(GAMMALN) composition (GAMMA and GAMMALN are independent), shifted variables (y−2,
  y−1), split-chain joins, split products S·(y−1)+S, a switch point (it is a smooth ramp), unstored final sums,
  directed rounding, constant / e-y term families, a value-changing correction near 2 (bias flat).

### 3.4 K3 state

- Structure: (x−1)·fl(N/D), (9,9) raw x; **plain stored** (noise fingerprint: real 1.001 vs stored 0.997; any
  mask leaving mid stages unstored is excluded; the top 2-4 stages are unresolvable).
- ss_fc sits at a local maximum of the near-null lattice (BKZ/CVP climbs: 0.333 → 0.341 held out).
- Per-stage phase reads (`w5/p5/k3phase.py`, all 16 sums): readable at N3, N5, N6, N7, D4, D5, D6; the laws are
  SMOOTH but STEEP (10²-10⁵ cycles per unit x), i.e. the true coefficients are far along null directions; the
  existing spacing cannot unwrap them.
- **Ladder probes** `kmyst-gam-k3l[a-f](00|10..21)` (78 cases, 640k points; anchors 0.8013, 0.9517, 1.1023,
  1.2519, 1.3527, 1.4531; base run plus runs at anchor + 2^-k, k = 10..21) are being filled (anchors a-b landed
  at handoff). The unwrap/solve tool `w5/p5/ladder.py` (+ `phaselib.py`) is validated on planted truths
  (slopes < 0.2%; TSIZE 1e6 within ~10%; TSIZE 1e7 works for D, fragile for N: add 6 anchors if needed).
- Chain after the ladders: slope solve → per-stage coefficient moves → peel (fscan ±2e6 + exact re-score) → N0.

### 3.5 (1.5,2] state (W6)

- Outer ops fl((x−1)·fl((x−2)·R)); R = N/D raw x; D0 is NOT a power of two (normalising by D0 created a phantom
  5e-15 contradiction; tools now use `synth.nrm` = normalise by ±2^k only).
- Phase-pinned upper stages (lines N.T4/T5/Pn14/15, D.T5/T6/Pn12-15) at control level; D.P2 lit at D3 −2,070,784
  ulps (Z 163); N4 +3 mod 8; low block at the noise floor with D0 fixed after the D3 fix; exact R 42.7%.
- N.P3 and D.P1 dark so far; alias scans (N4 ±1e6 at N.P3, D2 at D.P1 with the 512-ulp period below x ≈ 1.65)
  were running. Hidden c/x term: no effect (not the K2 mechanism here). Mask gains (51%) are suspect until the
  fingerprint is run (W5 was waiting on W6's definitions).

## 4. Methods that work (use them; they are in the repo)

- **Planted controls first, always.** Every search, scan and refit must recover a planted truth built from
  realistic coefficients before its real-data result counts. Several of today's "results" died on this rule.
- **Phase read** (`review3/phase.py`, `phase_stages.py`, `w5/p5/k3phase.py`): per-cluster calibrated sawtooth
  phase per stage; laws across x name null directions. Steep laws need **ladder probes** to unwrap
  (`w5/p5/ladder.py`).
- **Noise fingerprint** (W5's `w5/fp/`; W2's version is in W2's notes): LS fit of N − A*·D (A* = mean of doubles A
  with fl(t·A) = Y); residual rms in ulp(A) is the rounding noise of the true op structure, independent of
  coefficient bits. Calibrate with PLANTED truths with realistic coefficients (coefficient bits alone move it by
  ±0.03). Resolves large unstored regions only.
- **Null-lattice refinement** (fpylll LLL / BKZ near-null basis, greedy multi-scale, cluster objective + long-run
  holdout + cluster-mean-bias rms as a second objective).
- **Per-run caps** (constant offsets per stored op, Fact 1: within a 2e-12 run every coefficient move is a
  constant pre-rounding offset). Caveat: coordinate-ascent caps are local under-estimates; validate with a
  planted coefficient change.
- **c/x / c·g(x) head moves:** +c·g(x) on the head coefficients (g ≈ 1/x polynomial), −c on the next one, late
  fixed; scan c at step 1 (needles are ~2 units wide).
- **Model-free hand-off reads** (admissible s4 per point), **δ-profiles** at y and R, **class tables** (position
  × floor parity), **site maps** (fscan6 lit/dark with stored synthetic controls).

## 5. Traps (each cost hours today; do not repeat)

1. **Mask gains are not structure evidence.** Leaving ops unstored hides coefficient errors and lifts exact rates
   on wrong coefficients (shown on K1, K2, K3 with planted controls). Only the noise fingerprint, near-2 ladder
   tests or refits that transfer to unseen regions decide structure.
2. **Normalising by a non-power-of-two D0** changes the op sequence ((1.5,2]: phantom contradiction).
3. **LS re-solving the late chain** when testing head moves destroys the late rounding pattern Excel has; head
   moves must be compensated at the next constant only.
4. **Per-run sweeps that let the integer part of an offset reach the output** cannot represent odd-ulp/near-null
   moves.
5. **Column-letter extraction:** q501/q502 put GAMMA in column B; always select by formula (fixed
   `extract_runs.py`).
6. **Search ranges too small / steps too coarse:** K2's needle was 1-2 units wide and 1,200+ ulps out; K3's laws
   are 10²-10⁵ cycles/x.
7. **"Exact-rate" fits are over-tunable**: always hold out (alternate clusters, long runs, near-2 ladders).
8. **Local caps are not suprema** (K1 dip).
9. Workers' messages can cross; one reconciliation round per disagreement, with both sides scoring the same data.

## 6. Dead ends (do not retry without new evidence)

- Published coefficient sets; quadratic g; higher degree heads (N10, D9: emulable); common factors; Laurent
  c2/x², c3/x³; split c across product/constant; 80-bit/extended head constants (all coefficients are plain
  doubles); power tables, Estrin, even/odd, Dekker split, shifted variables (x−c) for K2 and K1; FMA anywhere
  (and FMA didn't exist on x86 in Excel's era); pure SSE2; input misparse / decimal transport (16-digit) / low
  bits of x / float32 tie deciders; exp(GAMMALN) for GAMMA; reciprocal-multiply or reordered final ops; a
  two-table K2 split at any of 64 points; low-precision reciprocal for the c/x term; fully-80-bit or "last k
  stored" chains.

## 7. Live hypotheses and next steps, ranked

1. **K2 structure (stored vs head-80)**: run the far-only-refit → near-2 transfer control (reviewer 3's plan).
   Then refit under the winner with the near-2 constraint and both objectives. Then attack the ~5% residual with
   the per-stage phase read re-based on the final model (which stage's line is still displaced, and what law).
2. **K3**: ladder unwrap (`ladder.py`) on k3l → per-stage slopes → coefficient moves (c·g-style head moves at the
   readable stages N3/N5-N7/D4-D6) → peel. Add 6 anchors if the N slopes reach ~1e6 cycles/x.
3. **K1**: global dip-weighted null-lattice search (BKZ basis ±1..±64, from plain1 and the dip-only set), then the
   D P2 line unwrap in the dip into Excel's S3(y) (ladder probes `kmyst-gam-k1l*` at anchors 1.80, 1.84, 1.87,
   1.89, 1.92, 1.96 if the dense runs don't unwrap).
4. **(1.5,2]**: noise fingerprint (needs W6's chain/mask definitions), alias scans at N.P3 / D.P1 with the right
   periods, then the c·g head-move recipe at the hand-off stages, and per-stage phase laws.
5. Cross-kernel: the same phase-read → law → plain-coefficient recipe should be applied stage by stage
   everywhere; the c·g(x) form found on K2 (cubic ≈ c/x) and K1 (degree 4-5 ≈ e/y) suggests Excel's tables were
   produced by a rational fit whose head was re-expanded (a near-null re-parametrisation); worth checking whether
   the K1/K3/(1.5,2] far offsets have the same "head += c·g(x), next −c" shape.

## 8. Data inventory

- Long runs (16,384 or 8,192 consecutive doubles): r1xx, r2xx, r6xx, r7xx, r9xx, p4xx, v4xx; near-2 ladders
  e4xx, e5xx, r204; dense clusters q201a..q216b (1,024 × 512); (1.5,2] runs w6t (306+/750 filled), L-runs, q44x
  ladders; K3: k3t (208+/450), rows6 builder `w5/p5/mkrows.py`, ladders k3l (78, filling); K1: k1R / `g_rows.py`
  (3.59 M unique points).
- Fill tools (coordinator scratch, not in repo): `fillmix.sh N` interleaves unfilled w6t/k3t; `fillgrp.sh REGEX
  MSG` fills a group and commits. Fill time grows superlinearly with case size: keep cases ≤ 16,384 rows.

## 9. Engine guidance (what to ship now)

See the engine task T-2123 (`docs/tasks/M9/T-2123-ship-t-1941-gamma-models-bit-exact-pieces.md`). In short: ship every solved
piece of section 2 bit-exact, and the best plain-coefficient models of section 3 for the four kernels (K2:
`k2_ref.py` / `k2_plain.txt` stored or masked per 3.2; K1: plain1; K3: ss_fc; (1.5,2]: W6's chain), each behind a
documented "not bit-exact" note, replacing the current fdlibm / statrs kernels. Expected bit-match: GAMMALN
(2,3] ~95%, GAMMA [1,2) ~78% (88% within 1 ulp), GAMMALN near 1 ~33%, (1.5,2] ~40%; everything else exact.

## 10. Restarting the search (the continuation task's prompt)

The continuation task is `docs/tasks/M9/T-2122-continue-t-1941-crack-the-four-open-gamma.md`; the engine task is
`docs/tasks/M9/T-2123-ship-t-1941-gamma-models-bit-exact-pieces.md`. Its "How to run" section is the restart prompt: one
coordinator plus one worker per kernel (K1, K2, K3, (1.5,2]) and one reviewer, all on this branch, each starting
from sections 3-7 here.
