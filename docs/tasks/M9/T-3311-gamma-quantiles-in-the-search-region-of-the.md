---
id: T-3311
title: "Gamma quantiles in the search region of the deep lower tail: 7 visibly-off gen-m9 cells left by T-2561 (CHISQ.INV(1E-200,5), (1E-20,6), GAMMAINV(1E-16,2.5,1))"
milestone: M9
lane: functions-stat
status: in-progress
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
- [ ] The seven cells reach visible parity, or the Outcome lists the candidates with their bit-exact and visible
      counts (decision 0056).
- [ ] No new known-failure line in any eval/baseline file; no cell T-2561 fixed gets worse.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells this task owns are visibly off, so it is in the campaign. Its target is that every visibly-off
cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value;
bit-level misses left then go to a `defer: phase-1b` follow-up.
For these seven cells Excel is the side off the true quantile; ours is the correctly rounded crossing. So
reproduce Excel's landing, never a more accurate kernel.

## Out of scope
- Shape 20 and above (T-3294); the p = 0.999 band (T-3146).

## Outcome
Two attempts (decision 0056); the second ships. Four of the seven targets now match Excel (A212 and A301 bit for
bit, A128 and A98 within 15 digits); A169, A338 and A355 stay visibly off and go to T-3403.

**Finding** (docs/excel-semantics/statistics.md, T-3311 paragraphs of "Deep lower tail at a shape above 1"; new
probes-numeric cases t3311-gamma-inv-lower-tail-cdf, 899 cells, and t3311-fresh-sweep, 240 cells, excel-graph):
Excel's GAMMA.DIST/CHISQ.DIST lower tail at a shape above 1 is T-3060's leading-factor series form, a staircase in x
(flat over ±4 ulp around every probed quantile), and GAMMA.INV/CHISQ.INV are Newton steps from the leading-factor root
on it. Excel's value is an exact iterate at most points; which iterate it returns once the steps cycle is not pinned.
`CHISQ.INV(p,2a)` and `GAMMA.INV(p,a,2)` are exactly twice `GAMMA.INV(p,a,1)` (88 of 88).

**Code.** First commit: T-3313's move of the closed-form quantiles into `incomplete_gamma/closed_form.rs`
(unchanged; T-3313 closed with a note). Then `closed_form::excel_newton_quantile`: after T-2561's root, up to 50
Newton steps from the root on `excel_form_lower` with `gamma_density`, keeping the first iterate whose cdf is
nearest p, times the scale. It applies only where the form at the root is within `NEWTON_START_MAX` = 1E-8 of p and
inside the measured range (shape 1.5 to 5, p 1E-200 to 1E-12); the search applies elsewhere. It covers either
function whose smaller tail is the lower one (`CHISQ.INV.RT(1-1E-16,3)`). R3 (CHISQ.INV.RT at p < 1E-150) is the
right tail, which this path never takes.

**Attempts.**
1. The cut at 1E-6. Fixes A169 and A338 as well, but moves gen-m9/c18dbf21f11093f3 A107
   `CHISQ.INV(POISSON.DIST(9,100,FALSE),8)` from bit-exact to 1 ulp off, a gated regression. No iterate is Excel's
   value there, and no density or form ordering tried changes that. Not shipped.
2. The cut at 1E-8, the narrow end of the train plateau (below). Shipped.

Rules tried for the iterate, on the 97 points of the first probe (exact / within 15 digits; engine 34 / 93):
- nearest-F over 15 to 50 steps: 37 / 94;
- nearest-F over 5 steps: 38 / 88;
- stop when |F - p| ≤ 1E-14 to 3E-14: 27 to 28 / 88;
- stop on a step ≤ 4E-15 to 2E-14 of x: 27 to 28 / 85 to 87;
- the same with the step taken: 22 to 24 / 85 to 86.

For Excel's cdf, 17 orderings of the form were tried: T-3060's is within an ulp at 304 of 330 cells (the engine's
pair: 46), and one variant reaches 307.

**R4 guards** (decision 0188). The model is structural (an algorithm family); `NEWTON_START_MAX` is a fit, tagged
`FITTED(T-3311, 13 cells, shape 1.5..5, p 1E-200..1E-12)`, and quality/fitted.json goes from 80 to 81 tags.
- Hold-out, disclosed: the split was fixed after an all-data exploration that chose the model, so it is not a clean
  hold-out. Split: FNV-1a of the cell key mod 100 < 30, giving 53 test and 100 train of 153 distinct (shape, p)
  points from gen-m9, probes, probes-numeric and the first probe. Margin: 15 points of exact rate, stated with the
  split. Oracle values are parsed with correct rounding; a first pass through serde_json without `float_roundtrip`
  misread some by an ulp, and its counts are discarded.
- Cut choice: lexicographic (within 15 digits, within 4 ulp, exact) on train. Cuts from 1E-8 to 1E-6 tie at
  (98, 97, 31) of 100. Within the tie, gated no-regression (A107) picks 1E-8; a reviewer may read that as a
  keep-matched choice. On test, 1E-6 would score 23 / 53 / 53.
- Frozen function (exact / within 4 ulp / within 15 digits):

  | candidate | train (100) | test (53) |
  |---|---|---|
  | HEAD (T-3311) | 31 / 97 / 98 | 21 / 51 / 51 |
  | correctly rounded (mpmath) | 20 / 88 / 94 | 10 / 44 / 46 |
  | main (engine) | 27 / 93 / 97 | 17 / 49 / 50 |

  Exact rate: train 31%, test 39.6%, within the margin. HEAD beats correctly rounded on test (21 vs 10 exact) and
  main (17). It changes 10 points and loses none: each is now bit-exact or 1 ulp closer.
- In-region (13 points): 10 bit-exact, 3 at 1 ulp.
  Over all points with the form at the root within 1E-6 of p (37): main 23 exact / 33 within 15 digits, the
  unrestricted steps 26 / 37. From 1E-6 to 1E-4 (14 points), the steps lose: 0 exact and 12 within 15 digits,
  against main's 2 and 14.
- Fresh sweep (R4.2): probes-numeric/t3311-fresh-sweep, 240 cells, generated and filled after the freeze. Shapes
  log-uniform 1 to 20 (df 3 to 39), lower-tail p log-uniform 1E-300 to 1E-3 (1E-15 to 1E-3 as `1-q` for
  CHISQ.INV.RT), 30 scaled.
  - Main: 97 exact / 156 within 4 ulp / 172 within 15 digits. HEAD: 102 / 160 / 176.
  - 5 cells changed, all to bit-exact, 4 of them visibly fixed (A46, A119, A212, A238; A11 was 2 ulp off); none
    worse.
  - Gate A envelope (within 15 digits of Excel, or 1E-9 of 50-digit mpmath): 240 of 240, main and HEAD; none
    outside.
  - Beyond the shipped range: of the 64 cells still visibly off, 58 are an exact Newton iterate, and the unrestricted
    rule would be bit-exact at 50 and within 15 digits at 62. That evidence is in T-3294's Context.
- Reproducibility floor: the cut is set against a 1-ulp regression (A107). That is below the 4-ulp web/desktop floor,
  so it is not fitted toward any target there; it only stops a loss.

**Eval comparison, main (c5c53f914 + claim) against HEAD, every cell:**
- Gated suites (only gen-m9 changes):
  - 5 fixed to bit-exact: A212 (-36 ulp), A301 (-11), 4e55500bb8ad3cd8 A111 `CHIINV(NPV(..),3)` (+5),
    58e3253bbd3a7a82 A287 and a7bf0d8d81d33ba6 A216 `CHISQ.INV(1E-16,3)` (-1 each).
  - 3 closer: A128 and A98 (-20 to -2 ulp, now within 15 digits), caa1eed48b1883fe A282 (-10 to -1).
  - Visibly fixed: 4 (A128, A98, A212, A301).
  - Lost bit-exact, farther, newly visibly off, type or error changed: 0.
- eval/baseline/gen-m9.txt: 5 F lines removed (`make eval-baseline`), none added.
- New t3311 probes (ungated, new in this PR):
  - Quantile cells: all improve.
  - Changes against main's engine on the same new cells: 26 bit-exact, 63 closer, 20 visibly fixed, 11 farther and
    5 newly visibly off. The farther and newly-off cells are all GAMMA.DIST/CHISQ.DIST evaluated at the changed
    quantiles; the engine's GAMMA.DIST is not Excel's staircase there.
- Other ungated suites: no change.

**Targets:**
- A212 `CHISQ.INV(1E-200,10)`: bit-exact.
- A301 `CHISQ.INV.RT(1-1E-16,3)`: bit-exact.
- A128 and A98 `CHISQ.INV(1E-200,5)`: 2 ulp, within 15 digits.
- Not reached (T-3403), each still visibly off at its main value:
  - A169 `CHISQ.INV(1E-20,6)`: 2.9E-7 at the root, past the cut.
  - A338 `GAMMAINV(1E-16,2.5,TRUE)`: 4.6E-7, past the cut; the second iterate is Excel's bit for bit.
  - A355 `CHISQ.INV(1E-20,10)`: 2.2E-4, where the steps are no better than the search.

**Other misses looked at:**
- A326 `CHISQ.INV(0.975,3)` and A168 `CHISQ.INV.RT(0.3,7)` (upper-tail p of 0.025 and 0.3; both within 15 digits,
  3.5 and 2.3 ulp): Excel's cdf at its value is a staircase too (sheet S rows 8 and 9), but the root is far there,
  so this model does not reach them. Not changed.
- A371 `CHISQ.INV.RT(-0.0001+1,3)`: Excel is 3,084 ulp from the true quantile. Its CHISQ.DIST.RT around it is flat
  (sheet S row 7), as for the p = 0.999 band. Excel's value is never a Newton iterate at right-tail p above 1E-5
  (0 of 14 cells), so it stays with T-3146.

Follow-ups: T-3403 (which iterate Excel returns; A169, A338, A355, A107). T-3294 gets the deep-tail evidence in its
Context.
