---
id: T-3311
title: "Gamma quantiles in the search region of the deep lower tail: 7 visibly-off gen-m9 cells left by T-2561 (CHISQ.INV(1E-200,5), (1E-20,6), GAMMAINV(1E-16,2.5,1))"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
The seven deep-tail gamma-quantile cells T-2561 left visibly off are within half a unit of Excel's 15th digit, or
time-boxed with the forms tried.

## Context
T-2561 matched Excel where the quantile is tiny: the leading-factor root `incomplete_gamma::excel_tiny_quantile`.
Away from that, Excel's value is neither the engine's crossing nor the root with Newton steps
(docs/excel-semantics/statistics.md, "Deep lower tail at a shape above 1"). The cells, from gen-m9 (excel-graph):
- gen-m9/06b4e8f5de409d5e A128 and gen-m9/623663cacea53bbc A98 `CHISQ.INV(1E-200,5)`: Excel 3.233407780583136E-80, ours 20 ulp
  below. The root is 236 ulp below, and our cdf at the root is 6.24E-14 off p.
- gen-m9/d8266bc15533702a A212 `CHISQ.INV(1E-200,10)`: Excel 5.210342169394733E-40, ours 36 ulp below, the root 111 below.
- gen-m9/a612e89321a6df9c A169 `CHISQ.INV(1E-20,6)`: Excel 7.829736048647269E-7, ours 5 ulp below.
- gen-m9/c2974c061384fdd9 A355 `CHISQ.INV(1E-20,10)`: Excel 0.0005210568413974334, ours 0.0005210568413974328.
- gen-m9/a5828d3925575713 A338 `GAMMAINV(1E-16,2.5,TRUE)`: Excel 6.436215297435573E-7, ours 6.436215297435568E-7.
- gen-m9/ecc87f80077959e0 A301 `CHISQ.INV.RT(1-1E-16,3)`: Excel 5.5854857570344876E-11, ours 11 ulp below.

Leads:
- At `CHISQ.INV(1E-16,3)` (a7bf0d8d81d33ba6 A216) Excel is one Newton step from the root on the leading-factor series
  form (T-3013's step rule), but that rule is 71 ulp off at A169.
- A probe of GAMMA.DIST at Excel's quantiles would show whether Excel lands on a crossing of its own cdf there.

Cells (the seven above, listed so `scripts/exit_owners.py` credits them; gap triage 2026-10-31):
- gen-m9/06b4e8f5de409d5e.case.json (1): Calc!A128
- gen-m9/623663cacea53bbc.case.json (1): Calc!A98
- gen-m9/d8266bc15533702a.case.json (1): Calc!A212
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A169
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A355
- gen-m9/a5828d3925575713.case.json (1): Calc!A338
- gen-m9/ecc87f80077959e0.case.json (1): Calc!A301

## Acceptance criteria
- [x] The seven cells reach visible parity, or the Outcome lists the candidates with their bit-exact and visible
      counts (decision 0056). Six reached; A355 to T-3403 with the candidates listed.
- [x] No new known-failure line in any eval/baseline file; no cell T-2561 fixed gets worse. One line (A107) added by
      owner ruling under decision 0083's allowance (decision 0197); no T-2561 cell worse.
- [x] `make check` green; reviewers approve (round 1's 1E-6 request taken, by owner ruling).

## Visible-parity stage
Decision 0083: the cells this task owns are visibly off, so it is in the campaign. Its target is that every visibly-off
cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value;
bit-level misses left then go to a `defer: phase-1b` follow-up.
For these seven cells Excel is the side off the true quantile; ours is the correctly rounded crossing. So
reproduce Excel's landing, never a more accurate kernel.

## Out of scope
- Shape 20 and above (T-3294); the p = 0.999 band (T-3146).

## Outcome
Two attempts (decision 0056). Shipped at the hold-out's cut, 1E-6 (owner ruling, decision 0197): six of the seven
targets match Excel (A212, A301 and A338 bit for bit, A128, A98 and A169 within 15 digits). A355 stays visibly off
(T-3403). The cost is gen-m9/c18dbf21f11093f3 A107, bit-exact to 1 ulp off, within 15 digits, under decision 0083's
allowance (below).

**Finding** (docs/excel-semantics/statistics.md, T-3311 paragraphs of "Deep lower tail at a shape above 1"; new
probes-numeric cases t3311-gamma-inv-lower-tail-cdf, 899 cells, and t3311-fresh-sweep, 240 cells, excel-graph):
Excel's GAMMA.DIST/CHISQ.DIST lower tail at a shape above 1 is T-3060's leading-factor series form, a staircase in x
(flat over ±4 ulp around every probed quantile), and GAMMA.INV/CHISQ.INV are Newton steps from the leading-factor root
on it. Excel's value is an exact iterate at most points; which iterate it returns once the steps cycle is not pinned.
`CHISQ.INV(p,2a)` and `GAMMA.INV(p,a,2)` are exactly twice `GAMMA.INV(p,a,1)` (88 of 88).

**Code.** First commit: T-3313's move of the closed-form quantiles into `incomplete_gamma/closed_form.rs`
(unchanged; T-3313 closed with a note). Then `closed_form::excel_newton_quantile`: after T-2561's root, up to 50
Newton steps from the root on `excel_form_lower` with `gamma_density`, keeping the first iterate whose cdf is
nearest p, times the scale. It applies only where the form at the root is within `NEWTON_START_MAX` = 1E-6 of p and
inside the measured range (shape 1.5 to 7.5, p 1E-200 to 1.0000000827E-10); the search applies elsewhere. It covers either
function whose smaller tail is the lower one (`CHISQ.INV.RT(1-1E-16,3)`). R3 (CHISQ.INV.RT at p < 1E-150) is the
right tail, which this path never takes.

**Attempts.**
1. The cut at 1E-8, the narrow end of the train plateau (below). Held through review round 1; replaced.
2. The cut at 1E-6. Fixes A169 and A338 as well, but moves gen-m9/c18dbf21f11093f3 A107
   `CHISQ.INV(POISSON.DIST(9,100,FALSE),8)` from bit-exact to 1 ulp off, a gated regression. No iterate is Excel's
   value there, and no density or form ordering tried changes that. Shipped by owner ruling (decision 0197).

Rules tried for the iterate, on the 97 points of the first probe (exact / within 15 digits; engine 34 / 93):
- nearest-F over 15 to 50 steps: 37 / 94;
- nearest-F over 5 steps: 38 / 88;
- stop when |F - p| ≤ 1E-14 to 3E-14: 27 to 28 / 88;
- stop on a step ≤ 4E-15 to 2E-14 of x: 27 to 28 / 85 to 87;
- the same with the step taken: 22 to 24 / 85 to 86.

For Excel's cdf, 17 orderings of the form were tried: T-3060's is within an ulp at 304 of 330 cells (the engine's
pair: 46), and one variant reaches 307.

**Review round 1 (fidelity REQUEST_CHANGES), resolved: the 1E-6 cut, by owner ruling.**
- The cut: 1E-8 was a keep-matched tie-break (it dodged gen-m9/c18dbf21f11093f3 A107); the review asked for the
  hold-out's choice, 1E-6, with A107 taken under decision 0083's standing ulp allowance. Its record (A107 as an F line
  via `update-baseline --allow-regressions`, listed in a decision record) was refused by this session's permission
  classifier ("[CI Bypass]") when committing the record, before any baseline change. Per the review's instruction
  for a refusal, the branch held 1E-8 until a ruling. The 1E-6 model was frozen and measured below.
- Owner ruling, 2026-10-31 (https://claude.ai/code/session_01WTJ7rq9tsQyspbybqNLk1a): "Allow, with records".
  Decision 0197 records A107 under decision 0083's allowance; `NEWTON_START_MAX` is 1E-6 with its range
  (`FITTED(T-3311, 23 cells, shape 1.5..7.5, p 1E-200..1.0000000827E-10)`), the unit test pins A338 bit-exact, A169
  and A107 within 15 digits; `update-baseline --allow-regressions` added the A107 line and removed A338's.
- A107 at 1E-6: Excel 7.92099982566666E-8, ours 7.920999825666661E-8, +1 ulp, within 15 digits. All four 0083
  conditions hold (numeric and within 4 ulp; nothing newly visibly off; 6 visibly fixed against 1 regression; T-3403
  names A107).

**R4 guards** (decision 0188). The model is structural (an algorithm family); `NEWTON_START_MAX` is a fit with a
`FITTED` tag (at 1E-8: 13 points where the steps apply, shape 1.5..5, p 1E-200..1E-12; at 1E-6: 23 points, shape
1.5..7.5, p 1E-200..1.0000000827E-10). quality/fitted.json rises by one (decision 0197).
- Hold-out (R4.1), disclosed as contaminated: the 100/53 split (FNV-1a of the cell key mod 100 < 30, 153 distinct
  (shape, p) points, docs/research/t3311/split_points.json) was fixed after the model family was chosen on all the
  data. Margin 15 points of exact rate. Fallback is main with T-3131's search. Exact / within 4 ulp / within 15 digits:

  | candidate | train (100) | test (53) |
  |---|---|---|
  | cut 1E-6 | 32 / 93 / 98 | 22 / 53 / 53 |
  | cut 3E-7 | 32 / 93 / 98 | 21 / 51 / 51 |
  | cut 1E-8 (round 1 hold) | 32 / 92 / 98 | 20 / 50 / 51 |
  | main | 28 / 88 / 97 | 16 / 48 / 50 |
  | correctly rounded (mpmath) | 20 / 83 / 94 | 10 / 42 / 46 |

  Ranked by within 15 digits, then within 4 ulp, then exact, the train part is best at 3E-7 and 1E-6; 1E-6 is the
  wider and leads on test. Exact rate at 1E-6: train 32%, test 41.5%, within the margin.
- Fresh sweep 2 (R4.2, the clean check): probes-numeric/t3311-fresh-sweep-2, 360 cells, generated after the 1E-6
  model was frozen (commit 63d859826, 2026-10-31T10:18:08Z) and filled 10:22:43Z to 10:22:55Z. A1-A240 span the full
  domain as sweep 1; A241-A360 are log-uniform over the 1E-6 fit's box. Region = where the steps apply, judged with
  mpmath (docs/research/t3311/score.py). Exact / within 4 ulp / within 15 digits / Gate A:

  | candidate | all 360 | region 30 |
  |---|---|---|
  | HEAD, cut 1E-6 (frozen, shipped) | 179 / 261 / 285 / 360 | 23 / 28 / 30 / 30 |
  | cut 1E-8 (round 1 hold) | 165 / 249 / 278 / 360 | 11 / 18 / 25 / 30 |
  | main | 148 / 231 / 268 / 360 | 1 / 9 / 23 / 30 |
  | correctly rounded (mpmath) | 19 / 122 / 191 / 360 | 1 / 9 / 22 / 30 |

  At 1E-6, 40 cells change against main, one of them farther: A346 `CHISQ.INV.RT(1-1.30E-14,4)`, -1 to -3 ulp
  (within 15 digits). At 1E-8, 22 change, none farther.
- Fresh sweep 1 (probes-numeric/t3311-fresh-sweep, 240 cells, filled after round 0's freeze), the same columns:
  HEAD at 1E-6 106 / 162 / 177 / 240, at 1E-8 102 / 160 / 176 / 240, main 97 / 156 / 172 / 240, correctly rounded
  20 / 90 / 125 / 240; region 5: 5 / 5 / 5 / 5, 2 / 4 / 5 / 5, 0 / 3 / 4 / 5, 1 / 3 / 4 / 5. None worse than main
  at either cut. Beyond the range: of its 64 cells visibly off at 1E-8, 58 are an exact Newton iterate and the
  unrestricted rule is bit-exact at 50 (T-3294's Context).
- Gate A envelope: 240 of 240 and 360 of 360 for HEAD at both cuts, main and the correctly rounded quantile; none
  outside. The 50-digit references and scripts are docs/research/t3311/reference.py (reference_split.json,
  reference_fresh-sweep.json, reference_fresh-sweep-2.json) and score.py (score_cut1e-6.out, score_cut1e-8.out).
- Reproducibility floor: the cut is not fitted toward any difference below 4 ulp; the 1E-6 choice rests on cells 2 to
  36 ulp apart and on visible parity.

**Eval comparison, main (origin/main bf7db78fb, with T-3131's search) against HEAD, every cell.**
- At 1E-6 (shipped), gated: 6 fixed (the 5 below and A338, -5 to 0 ulp), 4 closer (A128, A98, A282, A169 -5 to -4),
  6 visibly fixed (A128, A98, A169, A212, A301, A338), 1 lost bit-exact (A107: Excel 7.92099982566666E-8, ours
  7.920999825666661E-8, +1 ulp, within 15 digits; decision 0197), nothing farther, newly visibly off or of another
  type. Re-checked after merging origin/main 1af9bd6c6: `update-baseline --allow-regressions` against the round-1
  head (1E-8) shows gen-m9 fixed 1 (A338), regressions 1 (A107), no other suite changed; in-scope visibly off 2261.
- eval/baseline/gen-m9.txt against main: 6 F lines removed, 1 added (A107, decision 0197).
- At the 1E-8 round-1 hold, gated suites (only gen-m9 changes):
  - 5 fixed to bit-exact: A212 (-36 ulp), A301 (-11), 4e55500bb8ad3cd8 A111 `CHIINV(NPV(..),3)` (+5),
    58e3253bbd3a7a82 A287 and a7bf0d8d81d33ba6 A216 `CHISQ.INV(1E-16,3)` (-1 each).
  - 3 closer: A128 and A98 (-20 to -2 ulp, now within 15 digits), caa1eed48b1883fe A282 (-10 to -1).
  - Visibly fixed: 4 (A128, A98, A212, A301).
  - Lost bit-exact, farther, newly visibly off, type or error changed: 0.
- New t3311-gamma-inv-lower-tail-cdf probe (ungated, new in this PR), at 1E-8: quantile cells all improve; its
  GAMMA.DIST/CHISQ.DIST cells, evaluated at the moved quantiles, show 26 bit-exact, 63 closer, 20 visibly fixed,
  11 farther, of which 5 newly visibly off: W!C56, W!D56, W!E56, W!F56 and W!F31 (the 11 are listed in T-3412, which
  owns Excel's staircase GAMMA.DIST at shapes above 1). At 1E-6 the case is 252 of 899 bit-exact and 129 visibly off.
- Ungated fresh sweeps at 1E-6 (re-run on the shipped head): sweep 2 179 bit-exact and 75 visibly off of 360, sweep 1
  106 and 63 of 240, as in the tables. Sweep 2's one cell farther than main is A346 `CHISQ.INV.RT(1-1.30E-14,4)`,
  -1 to -3 ulp, within 15 digits.
- Other ungated suites: no change.

**Targets:**
- A212 `CHISQ.INV(1E-200,10)`: bit-exact.
- A301 `CHISQ.INV.RT(1-1E-16,3)`: bit-exact.
- A338 `GAMMAINV(1E-16,2.5,TRUE)`: bit-exact (the second iterate; the form at the root is 4.6E-7 off p).
- A128 and A98 `CHISQ.INV(1E-200,5)`: 2 ulp, within 15 digits.
- A169 `CHISQ.INV(1E-20,6)`: Excel 7.829736048647269E-7, ours 7.829736048647264E-7, 4 ulp, within 15 digits.
- Not reached (T-3403): A355 `CHISQ.INV(1E-20,10)`, 2.2E-4 at the root, where the steps are no better than the
  search; still visibly off at its main value.

**Other misses looked at:**
- A326 `CHISQ.INV(0.975,3)` and A168 `CHISQ.INV.RT(0.3,7)` (upper-tail p of 0.025 and 0.3; both within 15 digits,
  3.5 and 2.3 ulp): Excel's cdf at its value is a staircase too (sheet S rows 8 and 9), but the root is far there,
  so this model does not reach them. Not changed.
- A371 `CHISQ.INV.RT(-0.0001+1,3)`: Excel is 3,084 ulp from the true quantile. Its CHISQ.DIST.RT around it is flat
  (sheet S row 7), as for the p = 0.999 band. Excel's value is never a Newton iterate at right-tail p above 1E-5
  (0 of 14 cells), so it stays with T-3146.

Follow-ups: T-3403 (which iterate Excel returns; A355 visibly off, A169 and A107 bit-level, A107 as decision 0083's
ulp follow-up; Visible-parity stage), T-3412 (Excel's
GAMMA.DIST/CHISQ.DIST at shapes above 1 is the leading-factor form; the 11 probe cells). T-3294 gets the deep-tail
evidence and the pair-vs-form root-test note in its Context.
