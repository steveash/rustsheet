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

## 11. Log after the restart snapshot

- **08:00 (1.5,2], W6.**
  - A "low stages + final N/D in 80-bit" model gave 56.8% exact R. W6's own planted control refuted it:
    - lowfit80 residual on a planted MASK_80 truth: 5.83e-17;
    - on a planted all-stored truth: 1.31e-16;
    - on real data: 1.14e-16, which is stored-like.
  - So the low stages ARE stored, and the dark low sites mean wrong coefficients. Trap §5.1 again.
  - The honest best is 42.7% exact R (all-stored, `w6/data/xl_chain_d3ls`).
  - D0 ≈ 0.00164.
  - N8/D8 alias scans are running.
- **08:00 K1, W4.**
  - A dip-weighted global climb (BKZ basis, pairs, singles, held-out half) finds no set serving both the dip and
    [1, 1.8); plain1 is a local maximum.
  - Next: the k1l ladder unwrap.
- **08:00 K3, W5.** The first real ladder slopes (anchors a, b) are 1e5-1.5e6 cycles/x, so the true offsets are far
  (≥ 1e7 scale). W5 added rungs 22-26 (`k3l[a-f]2[2-6]`) and anchors g-l (`k3l[g-l]`), 138 cases.
- **Fill queue** (coordinator scratch `fillqueue.sh`):
  1. k3l a-f 00-21 (in progress);
  2. k3l a-f rungs 22-26;
  3. k1l (78, generator `w4/gen_k1ladder.py`);
  4. k3l g-l;
  5. then w6t/k3t.

## 12. Pause (2026-10-20 ~08:15): final worker states and pickup points

The owner paused the research track to save usage. All workers are stopped and every oracle that landed is
committed. Each worker's scripts are committed: W2 in `w2/` (with `w2/fingerprint/` and `w2/k3/`), W4 in `w4/k1r/`,
W5 in `w5/p5/` and `w5/fp/`, W6 in `w6/`, and reviewer 3 in `review3/` and `review3/final/`. Paths inside the
pasted scripts point at a scratchpad. Reviewer 3 uses the literal `SCRATCH/`. Edit them before running.

### 12.1 K2 (2,3]: structure now argued to be plain stored; residual is a per-run offset at N.s3
- **Reviewer 3's structure verdict:**
  - The near-2 ladders are e401-e403, e501-e504, e511, e512 and r204: 139k points with x−2 from 4e-16 to 5e-7.
  - They score 0.9953 with grid_a stored. Every single-op variant is worse:
    - N.s3 unstored: 0.9685;
    - N.P4: 0.938;
    - D.d2: 0.990;
    - FMA at any stage: ≤ 0.987;
    - pure SSE2 double: 0.2 points worse;
    - all-extended: 0.29.
  - Near 2 the product ops are insensitive to the coefficients, so these mask tests stand.
  - Head-80 (M\*) costs 3 points near 2. Its +1 point far from 2 is compensating coefficient error.
  - The N.s3 rounding line is present in Excel's residual with amplitude ratio ≈ 1. An exact s3 would have no such
    line.
  - Not done: the far-only-refit transfer control.
- **Reviewer 3's "cand9" set** (stored; p0, p6..p9 and q5..q8 as grid_a in `review3/spec.py`):
  - p1 `0x1.5e5b738276791p-16`, p2 `0x1.0ee2f118918cdp-9`, p3 `0x1.572bf86841a71p-5`, p4 `0x1.09fd1ad1d8fecp-2`,
    p5 `0x1.f0a8c053986d7p-2`.
  - q0 `0x1.03cadf2cc5614p-18`, q1 `0x1.52fc9914bdd53p-11`, q2 `0x1.5303658ef4486p-6`, q3 `0x1.bb43b464d3964p-3`,
    q4 `0x1.c515177eb1641p-1`.
  - How it was built: a cubic fit to the cluster bias curve, plus the near-2 constraint 16δp1 + 8δp2 + 4δp3 = 7290u
    with δp4 = −5209u and δp5 = +3128u, then a bit-match scan (`review3/final/cand9.py`, `clusterbias.py`).
  - p4's parity is pinned odd.
- **Coordinator check (my own runs, same data, stored sim, points in (2,3]):**

  | Set | Near-2 ladders | 14 long runs | q2 cluster files |
  |---|---|---|---|
  | cand9 (stored) | **0.9926** | 0.9332 | 0.9384 |
  | `k2_ref.py` (head-80 mask) | 0.9465 | **0.9498** | **0.9540** |

  Neither dominates. cand9 is the better set close to 2, and k2_ref is better everywhere else. Both are good
  starts.
- **Residual (reviewer 3):** a per-run constant pre-rounding offset c3 at the N.s3 sum, with a weaker analogue at D.d2.
  - A per-run c3 scan over all 1,825 runs (2.33M cells) lifts 0.9416 to 0.9549. Single runs gain up to 4.7 points
    (r901: c3 = −0.25 ulp(s3), constant over 16 sub-blocks, 0.957 → 0.988).
  - c3 is deterministic (551k duplicate x, no conflicts).
  - It is constant for Δx < 5e-5 and decorrelated by 3e-4 to 5e-4.
  - It is exactly 0 for x−2 < 1e-5, then winds faster as t = x−2 grows:
    - −0.125 at t = 1.1e-4 to 2.2e-4;
    - −0.25 at 2.6e-4;
    - −0.375 at 4.3e-4;
    - −0.69 at 8.4e-4;
    - −1.31 at 1.6e-3;
    - −1.9 at 2.0e-3.
  - Far from 2 it looks pseudo-random per run, plus a slow ±0.35 wave.
  - Excluded:
    - head-coefficient changes (the bias curve forbids a winding Δ3);
    - an exact or 54-64-bit s3;
    - rounding modes and FMA;
    - 64-bit constants;
    - lookup tables on binary grids of x or t (2^-8 to 2^-16);
    - frozen s1 residues;
    - oracle batch effects.
  - An A/x² term at the s3 level has the right winding magnitude and vanishes at 2. But R5's phase prediction does
    not match (coherence ≤ 0.34), and the near-2 ramp is too slow.
- **K2 pickup:**
  1. Fill reviewer 3's micro-ladders. These are new probe cases, not written yet.
     - At x0 ∈ {2.05, 2.25, 2.5, 2.75, 2.95}: runs of 256 consecutive doubles every 2e-5 across 3e-3, which is 750
       runs.
     - Plus x−2 ∈ [1e-5, 3e-3] every 1e-5, which is 300 runs.
     - About 190k + 77k cells, ≤ 16,384 rows per case.
  2. Unwrap c3(x) with `review3/final/c3all.py` and `c3table.py`, and test the laws A/x², A/x³, log, (x−2)^k and
     bit patterns.
  3. Fit D's q0..q2 on the near-2 ladders. D wants about −0.2 ulp(d2) there.
  4. Re-check the s4, P4 and s5 line phases under cand9 (`review3/phaseM.py`-style).
  5. Exclude the e404 glitch point x = 2.0000661560063.
- **W2's view:**
  - The R3-basin set in `k2_ref.py` is the committed reference. W2's alternative basin scores 0.9522.
  - The fingerprint (`w2/fingerprint/finger.py`, `k2fp2.py`) cannot separate the K2 structures. It needs ≥ ~0.05
    rms to tell structures apart, so the near-2 ladder test above is the better discriminator.

### 12.2 K1 [1,2): W4 final
- `w4/k1r/plain1.txt`: 0.776 exact, 0.878 within 1 ulp. My recheck is on 3.59M points.
- plain1 is a strict local maximum under three searches:
  - the upper-coefficient BKZ lattice (multiples ×1..×64);
  - the dip-weighted global climb (`gclimb.py`);
  - pairs and singles (`mclimb.py`).
- W5's fingerprint says K1 is stored, or at most has a small 80-bit head. The x87-top mask is excluded. In the
  [1.84,1.97] dip, real noise is not above planted plain1, so the dip is a coefficient problem.
- **Pickup:**
  1. Fill `kmyst-gam-k1l*` (78 cases, committed, no oracles).
  2. Unwrap the D P2 line per rung (`dtrack.py` is the base).
  3. Reconstruct S3(y). Fit D3..D7 with D2 pinned, then refit N.
- Data: `k1R.txt` (3.33M rows) is rebuilt from the oracles with W4's extractor. `forensics.py` shows the
  smallest N/D ulp change needed per region.

### 12.3 K3 [0.7,1.5): W5 final (details in `w5/p5/README.md` "PAUSED")
- Best set: `w5/p4/ss_fc.txt`, 33.37% exact on rows6 (5.68M points; q501/q502 excluded because they are GAMMA).
  A lattice climb gives +0.7% at most.
- Plain stored (fingerprint: real 1.025 vs stored 1.028; masks 0.34-0.86). W2 agrees.
- **Ladder slopes so far** (cycles/x; some may be aliased by 2^21 until rungs 22-26 land):
  - a (0.8013): N5 −8.1e5, N3 +5.6e5, D6 +2.4e5.
  - b (0.9517): N6 −8.8e5, N5 +1.5e5, N3 −2.7e5, D6 +4.1e5, D5 +8.7e5, D4 −9.6e4.
  - c (1.1023): N6 −2.8e5, N5 −1.7e5, N3 +8.5e5, D6 −6.1e4, D5 +5.0e5.
- **Pickup:**
  1. Fill the oracles in this order:
     - `kmyst-gam-k3l[a-f]2[2-6]` (30);
     - the one missing f rung;
     - `kmyst-gam-k3l[g-l](00|1[0-9]|2[0-6])` (108).
  2. From `w5/p5`, run `KMAX=26 MINRUNG=12 python3 ladder.py ../p4/ss_fc.txt real`, with the `control` run first.
  3. Apply the solved N4..N7 / D5..D7 moves.
  4. Peel: `peelstep.sh` for N j = 6..1, then D, then `n0scan.py`.
- **W2's K3 c/x needle scans** (`w2/k3/lscan.py`):
  - N5, D3, N4, D4, N6 and D2 are flat.
  - Remaining: N3, D5, N7, D6, N2, D1, N8, D7, N1, D8 (the default `STAGES`). Build `libk3s.so` from `k3s.c`.

### 12.4 (1.5,2]: W6 final (details in `w6/README.md` "Pick-up")
- Honest best: `w6/data/xl_chain_d3ls.txt`, all stored, 42.7% exact R.
- `xl_chain_80.txt` (56.8%) depends on MASK_80. Both W6's planted control (§11) and W5's fingerprint exclude
  MASK_80:
  - real minus sim, median: +0.18 against MASK_80, versus −0.076 for all-stored;
  - at most the very last product/sum can be 80-bit, which is at the resolution limit.
  - So the 56.8% lift is hidden coefficient error.
- **Pickup:**
  1. Run `DEPTH=3 PERRUN=256 python3 w6/pkscan.py RALL w6/data/xl_chain_d3ls.txt N 8 70000 1 30000`, and the same
     for D 8.
  2. If an alias lights, run `bump.py` and then `sitemap.py`.
  3. Run `pkscan` at N.P3 (CH N J 4 PER 8) and D.P1 (CH D J 2 PER 512).
  4. Then `lowsolve.py`, `scoreR.py` and `climb6.py`.
- Rall is rebuilt with `rows_all.py OUT 1.5 2` plus `rext.py`.

### 12.5 Unfilled probe cases at the pause
- Committed without oracles:
  - k3l a-f rungs 22-26 (30) and the one missing f rung;
  - k3l g-l (108);
  - k1l (78).
- The rest of the w6t (750 total, 312 committed) and k3t (220 committed) sets were generated locally and never
  committed. Their generators are committed; re-run them to regenerate.
- The fill command is in T-2122. Commit landed oracles per batch.

### 12.6 Engine implications (T-2123)
- K2: ship `k2_ref.py` as the default. It scores best over most of (2,3].
- cand9 is 0.99 within about 0.02 of 2, and t near 2 also feeds GAMMALN on [3,8) just above each integer.
  Switching to cand9 there is a measured option, but it is not Excel's algorithm. Decide it with the eval data.
- Nothing else changes for T-2123.

## 13. T-2122 run 1 (2026-10-22, single opus worker, no sub-sessions)

Scripts and logs of this run are in `docs/research/t1941/t2122/` on the research branch (main keeps only
`gen_k2micro.py`). The K2 scripts read `SCRATCH/cand9_PQ.npy` (cand9 of 12.1 as [P(10), Q(9)+pad], built from
`review3/spec.py` with p1..p5 and q0..q4 replaced) and `SCRATCH/c3all.pkl` (from `review3/final/c3all.py`); replace
`SCRATCH/` with a local directory as in section 12. Probe data stays on the research branch.

### 13.1 Oracles filled
- Every case committed without oracles at the pause is now filled: k3l a-f rungs 22-26 and the missing f21; k3l g-l
  rungs 00, 10-26; k1l (all 78). No probe case on the branch is left without an oracle from this run's lists.
- New cases, filled:
  - **K2 micro-ladders** `kmyst-gam-k2m[a-e][0-2]`, `k2mz[0-4]` (20 cases, `t2122/gen_k2micro.py`): §12.1 step 1
    exactly (anchors 2.05/2.25/2.5/2.75/2.95, a 256-double run every 2e-5 across 3e-3; near 2, a run at 2 + k·1e-5,
    k = 1..300).
  - **K3 rungs 27-30** at all 12 anchors `kmyst-gam-k3l[a-l](27..30)` (48 cases; `w5/gen_k3ladder.py`, now `KMAX`-aware).

### 13.2 K3: the ladder slopes were aliased twice; the offsets are ~1e10 ulps
- With rungs to 26 (`t2122/ladder_real_af26.log`), every slope moved by a multiple of 2^21 from the §12.3 values
  (e.g. N5@0.8013 −8.1e5 → −2.5e7). With rungs to 30 (`ladder_real_af30.log`) they moved again by multiples of 2^26
  and 2^28: N6 at anchors c-f is now a coherent smooth law (−3.04e8, −2.99e8, −2.88e8, −2.70e8 cycles/x). N3 and D5
  are stable across KMAX. The 6-anchor joint solve is still inconsistent (7/15 N, 6/11 D equations kept).
- Planted controls (`ladder_ctl*.log`, `ladder.py ... control`, seed 1):
  - KMAX=26, 6 anchors: fine at TSIZE 1e8; at 1e9 every slope above ~2.2e7 wraps by exactly 2^26 (the real data
    look like this).
  - KMAX=30, 6 anchors: recovers N4-N8 and D6-D8 at 1e9 and 3e9; fails at 1e10 (D chain lost, N7 11% off).
  - **KMAX=30, all 12 anchors, TSIZE 1e10: N4-N9 within 0.2%, D6/D7 within 0.05%, D8 ~20%, D5 unidentifiable.**
    This is the configuration to use on the real data (13.4).
- So the true K3 coefficients are ~1e10 ulps (relative ~1e-6) from ss_fc along near-null directions: far further
  than any lattice climb could reach, consistent with ss_fc being a strict local maximum.

### 13.3 K2: the per-run c3 is a sawtooth, not a winding law
- Reproduced: cand9 stored scores 0.9418 on all 2.18 M unique x in (2,3] (`k2_allscore_cand9.log`); `c3all.py`
  with the micro-ladders: 2,871 runs, 0.9417 → 0.9550 with per-run c3 (`c3all2.log`).
- `k2c3window.py` (5-run windows, offsets in 1/32 ulp including 0, so c3 ≈ 0 regions are visible) shows c3(x) on
  every ladder as a **sawtooth**: a monotone ramp of 0.5-0.75 ulp(s3) over 1-1.7e-3 in x, then a jump back of
  ~0.4-0.75 (not 1). Ramp direction is decreasing at s3; the N.p4 read is the same pattern rising 0 → 0.5 ulp(P4)
  (`k2stagewin_ac.log`; P4 is one binade above s3 for x < 2.75). Earlier "decorrelated by 3e-4" was the per-run
  argmax mixing informative and uninformative runs (`k2c3unwrap.py`: informative-only steps have coherence 0.85).
- **Refuted:** a smooth linear pre-rounding offset law at any single N op (s2, p3, s3, p4, s4) on 3e-3 windows
  (`k2lin.py`, planted control (−700, 0.3) recovered exactly as (−698, 0.297) at 100%; real best gains +0.004 to
  +0.015, below the per-window reads; `k2lin_s3.log`, `k2lin_sites.log`).
- Reading: the offset is periodic in x with period ~1e-3 (near 2) to ~1.7e-3, ramp-and-reset, with partial
  resets. That is what a rounding residue of a coarser intermediate looks like, not a coefficient move. Next:
  (1) read the ramp's exact period at each ladder (finer windows, W = 2) and test it against binary grids of
  s2/P3/x values and against ulp(P4)/ulp(s3) binade edges; (2) test a two-site model (s3 + p4 jointly per window);
  (3) cand9's near-2 strength makes it the base for every K2 test.

### 13.4 K3 real ladder, 12 anchors, KMAX=30 (`t2122/ladder_real_12a30.log`)
- Still inconsistent: 10/27 N and 7/20 D equations kept, and the solved moves equal the 6-anchor ones (the RANSAC
  consensus is the same subset). Several dropped residuals sit near multiples of 2^28-2^29 cycles/x, so some reads
  are still aliased at rung 30 or are wrong-branch unwraps.
- The most coherent law in the data is **N6: −2.7e8 to −3.04e8 cycles/x at all 12 anchors from 0.73 to 1.45**
  (nearly constant in x), and RANSAC drops most of those equations. N3 (−1.6e5 to −1.6e7) and D5 (−4.6e7 to +3.8e7,
  smooth in x) are stable across KMAX 26/30.
- Next steps, in order:
  1. Do not trust the joint solve's consensus. Fix N6 from its 12 coherent equations first, then solve the
     remaining stages conditionally (pass `STAGES` without N6 after applying its move; or weight equations by
     rung count).
  2. Test the near-constant N6 slope directly: a pure move along the direction that makes δ6' constant
     (N7 and the N8/N9 compensation) at about −2.9e8 cycles/x, scored on rows6.
  3. If residuals stay at 2^28-2^29 multiples, add rungs 31-33 (`KMAX=33 python3 w5/gen_k3ladder.py ...`) for
     the 6 anchors whose reads disagree and confirm with a 1e11 control.

### 13.5 Not reached this run
- K1: the D P2 line unwrap (`w4/k1r/dtrack.py`) is not run.
- (1.5,2]: the w6t/k3t cases beyond the committed ones must be regenerated and filled before W6's pkscan, which
  is hours of CPU per stage; not started.
- No kernel is bit-exact yet, so `gamma_kernels.rs` has no new arrays to swap (no engine task filed this run).

## 14. T-2122 run 2 (2026-10-22, single opus worker, ~40 minutes, K3 only)

Script `t2122/k3_n6first.py` (main and the research branch; logs `k3_n6first_*.log` on the research branch). It
reads the per-stage slopes `w5/p5/ladder.py` prints and solves the N chain top-down instead of by the joint RANSAC
(13.4 step 1): N6's equations alone give (dN7, dN8, dN9), with a consensus start over minimal subsets and alias
shifts of 2^maxrung cycles/x; N7's reads are then a check; N5 adds dN6; N3 adds dN4 and dN5. With `JOINT67=1` it
also fits N6 and N7 jointly, enumerating alias shifts on the N7 rows only. It needs only numpy (no `libsc.so`). No
new oracle data this run.

### 14.1 Results
- **N6 is one smooth quadratic law at 8 of the 9 readable anchors.** In coefficient units, slope·ulp(S6) runs
  −3.83e-9 (0.8741), −4.19e-9 (1.0309), −4.22e-9 (1.1023) down to −3.74e-9 (1.4531). S6 stays in one binade
  (ulp 2^-56) over [0.73, 1.46], so the near-constant cycles/x read of 13.4 is not a binade artefact.
  - Fit on 8 anchors: residuals ≤ 2e6 cycles/x (0.7%).
  - The 0.9517 read (−7.6e6) is off by −2.83e8, about 2^28. It is an alias at rung 28 or a wrong-branch unwrap.
- **Anchors a-f alone are fully consistent.** `ladder_real_af30.log` gives 4 N6 equations and 2 N7 equations. The
  joint N6+N7 fit leaves rms 2.4e4 cycles/x, with N7@1.3527 at alias −1 (one period of 2^30). The moves are **N7
  +4.005e8, N8 −4.61e10, N9 +3.502e12 ulps** (`k3_n6first_real_af.log`).
- **With all 12 anchors the joint N6+N7 fit breaks.** Rms is 1.25e7 cycles/x and N7 residuals reach 3.7e7
  (`k3_n6first_real.log`). The N6-only 12-anchor fit moves N7 to +7.5e8 (+5.6e8 jointly). So the g-l anchors
  (0.7311 to 1.4129) pull against a-f by ~2e8 ulps in N7.
- The lower stages are not consistent yet: N5 has 3 readable anchors, 2 dropped; N3 keeps 6 of 10 at a 3e6
  tolerance, giving N4 −3.8e8 and N5 +1.04e9 ulps. They depend on the head move, so they mean nothing until N7-N9
  are settled.
- **Controls:**
  - On the 12-anchor planted control (`ladder_ctl30_1e10_12a.log`), N6-only recovers truth N7 to 1% (−1.33e9
    against −1.344e9). It does not recover N8 or N9 (−1.5e9 against −1.08e9; 3.9e10 against 4.3e9): N6's
    equations fix N7 and only a combination of N8 and N9.
  - The control's N7 reads miss the N6 prediction by 2e6-1.4e7. So N7 reads are good to ~1e7 cycles/x at best, and
    the real 12-anchor N7 misfit is only 2-3 times that.
  - **The `JOINT67` fit fails its control** (rms 2.7e8). It enumerates alias shifts on N7 rows only, and the
    control needs one on N6 (1.1789, alias −1). Its a-f real result passed because there no N6 row needed a shift.
    Fix this before trusting any 12-anchor joint number.

### 14.2 Next steps (in order)
1. Let the joint fit enumerate alias shifts on every row: ±1 on all, or a RANSAC over (rows, shift). Validate it on
   `ladder_ctl30_1e10_12a.log` until it recovers truth N7 and the N8/N9 combination.
2. Score the a-f head move (N7 +4.005e8, N8 −4.61e10, N9 +3.502e12 on ss_fc) against the 12-anchor one on the
   rows6 data. This is 13.4 step 2 and needs `w5/libsc.so` built: `make` in `w5/`.
   - If a-f wins, re-read the g-l ladders: the second-batch anchors are the suspect, rung by rung.
   - If neither wins, the head offset is not a coefficient move. That would be the same reading as K2's
     sawtooth (13.3).
3. Only after the head is fixed: N5 → dN6, then N3 → (dN4, dN5), each with its alias shifts free.

## 15. T-2122 run 3 (2026-10-22, single opus worker, ~30 minutes, K3 step 14.2.1)

`t2122/k3_n6first.py` gains `JOINTALL=1`: the joint N6+N7 fit for (dN7, dN8, dN9) with alias freedom on every row.
It runs a RANSAC over every minimal 3-row subset times shifts {-1,0,+1} on each, refines each start (LS, then the
nearest alias per row), keeps a row only within `TOL` (default 3e6 cycles/x) and at |alias| <= `MAXALIAS` (default
2), and ranks solutions by rows kept, then total |alias| on kept rows, then rms. Logs: `t2122/k3_jointall_ctl.log`,
`t2122/k3_jointall_real.log`. Needs only numpy (a scratch venv works; the tools venv has none).

### 15.1 Results
- **The planted control passes** (`ladder_ctl30_1e10_12a.log`): 12/15 rows kept, rms 2.85e5 cycles/x, **N7
  -1.344e9 (truth -1.344e9)**, N8 -1.116e9 (truth -1.08e9), N9 +7.96e9 (truth 4.3e9; N9 is the weakly determined
  direction, as in 14.1). The one alias it needs is N6@1.1789 = -1, the row JOINT67 could not shift.
- **Degenerate family:** equal-rms solutions differ only in N7, by steps of about 4.3e9 ulps (2^32: a uniform alias
  on the N6 rows is itself almost a move of N7). Ranking by least total alias picks truth on the control. Do not
  rank by rms alone (the first JOINTALL draft picked N7 -5.6e9 that way, every N6 row at alias -1).
- **Two traps fixed while building it:** (1) the refine step, unbounded, drifts into lattice fits with |alias| 50-200
  that keep rows at any move size: hence `MAXALIAS`; (2) deduplicating solutions by their alias vector alone lets a
  low-rms 5-row fit overwrite a 12-row fit with the same aliases: key by (kept mask, aliases on kept rows).
- **Real 12-anchor data (`ladder_real_12a30.log`): one head move fits a-f and g-l together.** 10/14 rows kept at
  rms **2.65e4 cycles/x** (ten times tighter than the control's noise), moves **N7 +4.007e8, N8 -4.611e10, N9
  +3.503e12 ulps**: the a-f-only move of 14.1 (+4.005e8, -4.61e10, +3.502e12) to 3-4 digits. Kept: every N6 row
  but 0.9517 and 0.8741, and N7 at 1.3527 (alias -1), 1.4531, 1.4129. Dropped: N6@0.9517 (-2.9e8, the known 2^28
  alias/wrong branch of 14.1), N6@0.8741 (-8.4e6), N7@1.1789 (+2.6e7), N7@1.3021 (+5.9e7, alias -1). The control
  shows N7 reads good only to ~1e7, so the N7 drops are 2.6 and 6 times that noise, not evidence against the move.
  TOL 1e6 gives the same solution; TOL 1e7 keeps 12/14 and moves N7 to +4.99e8 (rms 4.5e6), i.e. the dropped rows
  pull by ~1e8 ulps at most.
- So run 2's "g-l pull N7 by ~2e8" was the missing N6 alias freedom, not a second-batch anchor problem.

### 15.2 Next steps (in order)
1. 14.2 step 2 with this move: score N7 +4.007e8, N8 -4.611e10, N9 +3.503e12 (on `w5/p4/ss_fc.txt`) against the base
   on the rows6 data (`make` in `w5/` for `libsc.so`). Also score the TOL=1e7 variant (+4.99e8, -4.96e10, +3.81e12)
   as the sensitivity check.
2. Re-read N6@0.8741 and N7@1.1789/1.3021 rung by rung (they are the only rows that disagree beyond noise).
3. Then 14.2 step 3 (N5 -> dN6, N3 -> dN4, dN5) with the head fixed; give those fits the same all-row alias freedom
   and least-alias ranking (port JOINTALL's search into `fit()`).

### 15.3 Step 15.2.1, first score (same run)
Rows: `w5/p5/mkrows.py` now gives **9,606,590 points from 741 cases** (rows6 was 5.68 M). Built `w5/libsc.so`
and `w5/p5/libhypo.so` with the gcc lines in `sc.c`/`hypo.c` (not committed). Moves applied by `t2122/k3_move.py`
on `w5/p4/ss_fc.txt`, then `w5/p5/refit_fn.py IN p4/ss_fc.txt OUT N 7 [EXCL]` restores the function; scored by
`t2122/k3_score.py` (exact rate: all / |x-1|>2e-3 / near 1).

| set | refit freedom | all | far | near 1 |
|---|---|---|---|---|
| ss_fc (base) | none | 0.3217 | 0.3052 | 0.7294 |
| zero move (control) | all 16 (N0-N6, D1-D9) | 0.2595 | 0.2565 | 0.3343 |
| A: N7 +4.007e8, N8 -4.611e10, N9 +3.503e12 | all 16 | **0.2983** | 0.2944 | 0.3933 |
| B: N7 +4.986e8, N8 -4.96e10, N9 +3.812e12 (TOL=1e7) | all 16 | 0.1562 | 0.1462 | 0.4004 |
| zero move | N0-N6 only (D fixed) | 0.2894 | 0.2830 | 0.4464 |
| A or B | N6 only, or N0-N6 | 0.0000 | | (function off by 3e-9 / 1e-12) |

- **The refit is not neutral:** with no move at all it costs 6 points (0.322 -> 0.260). Score moves against that
  control, not the base. A beats it by +3.9 points (+5.9 near 1); B loses 10. So the least-alias joint move (A) is
  favoured over the TOL=1e7 variant, but no protocol here beats the base yet.
- A's move needs D-chain freedom to stay function-equivalent: N6-only or N-only compensation leaves the function off
  by 3e-9 or 1e-12, and every point fails. So "compensate at the next constant only" does not apply to a move this
  size; the head move and the lower chain must be solved together.
- `t2122/k3_moveA_refit16.txt` is set A after the 16-coefficient refit.
- Next: a refit that keeps the base's lower-chain bits where it can, for example the 15.2 step 3 conditional
  solves (N5 -> dN6, N3 -> dN4, dN5, then D) instead of a least-squares function refit. Score it against the zero-move
  control of the same protocol.

## 16. T-2122 run 4 (2026-10-23, single opus worker, K3 steps 15.2.3 and 15.3)

Script `t2122/k3_lower.py`: JOINTALL's search (all-row alias freedom, least-alias ranking) for any set of stages
with the moves above them held fixed. `PLAN` lists the steps as `CH:stages:moves`; `DD9` pins D9 (default 0).
`t2122/k3_move.py` now also takes `KEY=ulps` moves on any coefficient. Logs: `k3_lower_ctl.log`,
`k3_lower_real.log`, `k3_lower_real_topdown.log`, `k3_refit_noise.log` (`k3_refit_noise.sh`). Needs numpy only, plus
`w5/libsc.so` for scoring.

### 16.1 Results
- **Strict top-down fails its control.** On `ladder_ctl30_1e10_12a.log` N5 has no readable rows, so N3's solve
  runs with dN6 missing and gives N4 +6.9e8, N5 +2.3e9 (truth +3.60e8, +5.09e9).
- **Joint N3+N5 with the head fixed passes it.** With the run 3 control head (N7 -1.344e9, N8 -1.116e9, N9
  +7.96e9), `PLAN=N:3,5:4,5,6` keeps 5/5 rows at rms 1.8e3 cycles/x and gives N4 +3.600e8, N5 +5.088e9, N6
  -4.881e8. Truth is +3.598e8, +5.090e9, -4.889e8 (all within 0.2%).
- **The D chain stays unidentified below D6, as in 13.1.** D6's rows give D7 and only a combination of D8 and D9.
  D5's rows then fix D6 to 0.3% (-1.277e9, truth -1.281e9). D7 is 4% off with D9 pinned at 0 (truth -1e10). D8 is
  wrong (+1.1e9 against -1.37e8). D4 keeps 1 row of 5, so D5 is unreadable. Joint D5+D6 with D9 free gives D9
  -8e14: the direction is null.
- **Real data with head A (N7 +4.007e8, N8 -4.611e10, N9 +3.503e12):**
  - Joint N3+N5 keeps all 10 N3 rows and N5@0.9517 at alias 0, rms 4.45e5 cycles/x. Moves: **N4 +3.87e7, N5
    -1.603e9, N6 +7.37e8**. N5@0.8013 and N5@1.1023 are dropped at +1.7e8 and +3.7e8.
  - This agrees with ladder.py's 12-anchor joint consensus of 13.4 (N4 +7.2e7, N5 -1.85e9, N6 +7.74e8, N7
    +4.02e8, N8 -4.616e10, N9 +3.5075e12). The whole N chain, N4 to N9, is now one consistent move.
  - Top-down (N5 alone, then N3) keeps 1/3 N5 rows (N6 +3.98e8) and 7/10 N3 rows. That is the control's failure
    mode, so do not use it.
  - Real D6 keeps only 4/10 rows (D7 -6.3e9, D8 +1.7e11). D5 keeps 8/8 and gives D6 +4.5e8, but that is
    conditional on the D6 solve.
- **The 15.3 scoring protocol cannot rank moves this size (`k3_refit_noise.log`).** Refit N0-N3 + D1-D9 to ss_fc's
  function and score on the 9.6 M rows. A one-ulp change in N6 then moves the exact rate by up to 10 points:
  - with the head + N4-N6 move: 0.297 (+0 ulps), 0.290 (+1), 0.271 (+10), 0.244 (+100), 0.212 (+1e3), 0.242 (+1e4),
    0.287 (+1e5);
  - with the zero move: 0.268, 0.266, 0.235, 0.283, 0.271, 0.274, 0.177;
  - the means are 0.263 against 0.253, inside the spread.
  So run 3's "A beats the zero-move control by +3.9 points" is within the protocol's noise and is **not evidence for
  A**. The scores of A + N6 only (0.210 against control 0.169) and A + N5 + N6 (0.231 against 0.243) are noise too.
  The cause is that the least-squares refit sets the low bits of 13-16 coefficients to whatever restores the
  function, and those bits decide the exact rate.

### 16.2 Next steps (in order)
1. Replace the refit-and-count score with one that is invariant to the lower chain's low bits. For example,
   per-run caps (section 4, Fact 1) on the moved set, with only the head and N4-N6 fixed. Or re-read the per-stage
   ladder slopes on the moved set: a right move leaves N3-N7 slopes at about 0 cycles/x. First calibrate the score
   on the planted control: truth against the zero move against a 1-ulp perturbation.
2. If you keep a refit score, use an ensemble: score each candidate over 20+ tiny perturbations (as in
   `k3_refit_noise.sh`) and compare distributions, never single numbers.
3. The D chain needs more readable stages before any D move counts. Add rungs or anchors where D4/D6 are unreadable
   (0.9517, 1.0309, 1.1023, 1.4531), or use the per-run cap score to pick D moves along the null direction.

## 17. T-2122 run 5 (2026-10-28, single opus worker, ~50 minutes, K3 step 16.2.1)

Scripts: `t2122/k3_reread.py` (tabulates ladder logs made with a moved set as base; a row is "at zero" when it has
≥ 10 rungs and |slope| < 500 cycles/x), `t2122/k3_null.py` (SVD of the N-chain slope equations of the rows at zero:
the move the ladder cannot see), `t2122/k3_tol.py` (exact-rate tolerance of one-coefficient moves on a planted
truth). Sets `t2122/k3_run5_{A,B,C}.txt` (real) and `k3_run5_ctl_*.txt` (control: truth = `make_truth` TSIZE 1e10,
seed 1, kmin 15 on ss_fc, i.e. the truth of `ladder_ctl30_1e10_12a.log`). Logs `ladder_real_run5_*.log`,
`ladder_ctl_run5_*.log`, `k3_reread_{real,ctl}.log`, `k3_null_{real_C,ctl_c7}.log`, `k3_tol_ctl.log`.
Environment notes: the `.so` scorers are not committed; build them with the `Build:` line at the top of `w5/sc.c`
and `w5/p5/{hypo,inj,peelk}.c`. The system python has no numpy; use a venv.

### 17.1 The re-read score (16.2 step 1): ladder.py with the moved set as BASE
- It reads the **residual** slope of every stage, and it depends only on the coefficients above the stage, so the
  low bits of the lower chain (15.3's noise) do not enter. Calibrated on the control (`k3_reread_ctl.log`):
  - base = truth N (D still ss_fc), or truth with N6 +1 ulp: 15/25 and 22/28 rows at zero, rms 52 and 36 cycles/x.
    The other rows mis-unwrap by near multiples of 2^24 (e.g. +16777298); they are few-rung or isolated rows.
  - base = the control's own run-4 solve (c2, 0.2% off): N3 rows read +241 to +10463, N6 rows ~+1500, and each
    read matches that base's true residual slope to ≤ 1% (e.g. N3@0.9517 +9487.6 against truth +9465.2).
  - base with N5 +1e7 ulps wrong (c3): every N3 row reads about −1e6, matching its truth.
  So the score separates right from wrong moves, and the reads are themselves the next correction.
- **Newton iteration (re-read, solve the residual with ladder.py's joint solve, apply, repeat) converges.**
  Control: c2 → c6 → c7. The residual truth after c7 is N4 +3.6e3, N5 −3.2e4, N6 +5.9e3, N7 −8, N8 −1.6e4,
  N9 +1.9e6 ulps, and every row of c7 reads at zero, so this is the ladder's floor, not a failure to converge.

### 17.2 Real data: head move A plus N4-N6 is right, and two more steps reach the floor
- Re-read against A (run 4's head + N4-N6 move): N6 rows fall from ~3e8 to −5.4e4 … −2.4e4 cycles/x and N3 rows
  from ~1e7 to ±7e5. Both follow smooth laws in x, the c2 signature. (Base ss_fc: no row at zero.)
- Step 1 (A → B): N4 +3.34e7, N5 −2.49e8, N6 +3.74e7, N7 +8.1e5, N8 −3.05e7, N9 +2.41e9. 10/28 rows at zero.
- Step 2 (B → C, the solve printed in `ladder_real_run5_B.log`): N4 −9.46e3, N5 +1.24e5, N6 −2.64e4, N7 −9.2e3,
  N8 +4.06e5, N9 −3.80e7. **C: 21/29 rows at zero (rms 177 cycles/x), all 11 N3 rows within ±150.** C's own solve
  (N4 +647, N5 −3.8e3, N6 +401, N7 +5.9e3, N8 −2.8e5, N9 +2.7e7) is inside the 17.3 error bars; it is not applied.
- **C's total move from ss_fc: N4 +7.207e7, N5 −1.8522e9, N6 +7.742e8, N7 +4.0150e8, N8 −4.6140e10,
  N9 +3.50537e12 ulps.** Run 1's independent 12-anchor consensus (13.4) was N4 +7.2e7, N5 −1.85e9, N6 +7.74e8,
  N7 +4.02e8, N8 −4.616e10, N9 +3.5075e12: the same move.
- The D chain is still unsolved: 0/22 D rows at zero against C; the joint D solve returns D9 +5.7e15 (null).

### 17.3 What the ladder cannot see, against what the exact rate needs
- `k3_null.py`, read noise σ = 100 cycles/x. The 1σ error per singular direction in ulps (control c7 / real C):
  dir0 (N4,N6,N7 mix) 71 / 40; dir1 (≈ N7) 165 / 131; dir2 (N4−N6) 712 / 203; dir3 (≈ N8 on control, ≈ N5 on
  real) 3.2e3 / 1.7e3; dir4 (≈ N5 on control, ≈ N8 on real) 7.4e4 / 3.2e3; dir5 (≈ N9) 6.9e6 / 4.5e6.
  The control's true residual projects at 0.2-1.8σ on each, so σ = 100 is a fair noise model.
- `k3_tol_ctl.log` (planted truth, one coefficient moved, exact rate): N9 +389 → 0.9967, +1e4 → 0.915,
  +1e5 → 0.47; N8 +3 → 0.995, +10 → 0.983, +100 → 0.83; N7 +1 → 0.959, +10 → 0.62; N5 +1 → 0.90, +10 → 0.50;
  N4 +1 → 0.63.
- So C is still 2-4 orders of magnitude too coarse on N9, N8 and N5, and 1-2 on N4, N6 and N7. Its exact rate is
  not a useful measure yet (lambda_path in `w5/p5/README.md`: no partial credit outside ±1-2 ulps).

### 17.4 Next steps (in order)
1. More ladder rows on the weak directions: N7 is readable at only 1 real anchor (1.4129) and N5 at 1 (0.9517).
   Add anchors where N7 and N5 read (N7 is readable above 1.17 on the control; see the `N7:` lines of
   `ladder_ctl_run5_c4.log`) and rungs 31-33 (`KMAX=33 python3 w5/gen_k3ladder.py`). Then re-run the Newton step from
   C. Each 1σ shrink is a factor of fewer hypotheses in step 2.
2. Close the top with W5's exact tools from C: `topgrid2.py` (N8, N9 grid on the S-only objective; widen R8 to
   about ±3σ of dir4 and R9 to ±3σ of dir5, or search along the SVD directions instead of N8/N9). Then `fscan.py`
   (±2e6 FFT peel) for N6 … N1 and `n0scan.py` for N0, as in `e2e_hchain.sh`. Run the same chain on the c7
   control first; it starts at the same distance from truth.
3. The D chain: run the same Newton iteration with D9 pinned (`k3_lower.py`'s `DD9`) once ≥ 6 D rows read at
   zero; until then, add anchors where D5/D6 are readable.
