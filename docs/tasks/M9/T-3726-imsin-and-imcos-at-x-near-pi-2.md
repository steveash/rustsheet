---
id: T-3726
title: "IMSIN and IMCOS at x near pi/2 with a large y: the cos x part 7E-6 relative off (t3616 K1, L1, K22, L22)"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
IMSIN and IMCOS give Excel's text when the real part is near pi/2 and the imaginary part is large, so their
small part (cos x · sinh y for IMSIN, -cos x · cosh y for IMCOS's real part) matches Excel in all 15 digits.

## Context
probes-numeric/t3616-imsin-imcos-positive-cut-probe (excel-graph, T-3616), x = 1.5707963267949 (cos x is about
3.49E-15, so only a few digits of x past pi/2 survive):
- S!K1 `IMSIN("1.5707963267949+709.7i")`: Excel 8.27492013840132E+307-2.88915216855774E+293i, ours
  ...-2.88917292436455E+293i; K22 (y = 709.699999999999) the same shape.
- S!L1 `IMCOS("1.5707963267949+709.7i")`: Excel -2.88915216855774E+293-8.27492013840132E+307i, ours
  -2.88917292436455E+293...; L22 likewise.
Ours is 7.2E-6 relative off in the small part only; the large part matches. complex/trig.rs `imsin`/`imcos` take
`x.sin()`/`x.cos()` from libm. 7E-6 relative at a remainder of 3.5E-15 is an absolute error near 2.5E-20, the size of
a reduction with a 66-bit pi (x87 FSIN/FCOS), so Excel's IM* trigonometry may reduce x as the x87 does rather than as
SIN/COS do (`excel_sin_cos`, math_and_trigonometry/trig.rs, which IMSIN of a real number already uses). Check
which candidate (libm, `excel_sin_cos`, x87 66-bit-pi reduction) fits these 4 cells and keeps every other IM* cell,
then confirm on a fresh sweep of x near multiples of pi/2 with moderate and large y.
Needs an oracle sweep of x near k·pi/2 (k = 1..4, both signs) with |y| from 1 to 709: draw it after the
candidate is frozen.

## Acceptance criteria
- [x] The 4 cells above match; no other IM* cell changes (fit labs `im-hyperbolic`, `im-hyperbolic-t3589-fresh`,
      `im-hyperbolic-log`, every IM* case in probes and probes-numeric).
- [x] Decision 0188 R4 guards (reviewer-excel-fidelity check 7): hold-out split fixed before scoring, a fresh sweep
      drawn after the freeze, Gate A envelope result, `FITTED` tag and range if a value is fitted, the reproducibility
      floor, and no keep-matched branch.
- [x] The rule is in docs/excel-semantics/engineering.md.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of
Excel's value. Bit-level criteria still unmet then move to a follow-up task named in the Outcome.

## Out of scope
- The `#NUM!` cut at y > 709.7 (T-3616).
- COSH/SINH last bits near 709.7 (T-3727).

## Outcome
- complex/trig.rs: IMSIN and IMCOS of a complex x + yi take sin x and cos x from `trig_sin_cos` (SIN and COS's own
  kernel, x reduced by the x87 64-bit pi below 2^27, libm beyond) instead of libm. K1, L1, K22 and L22 of
  probes-numeric/t3616-imsin-imcos-positive-cut-probe now match (case 320 -> 324 of 330; the 6 left are COSH/SINH last
  bits, T-3727). Unit test `imsin_imcos_near_half_pi_take_the_x87_reduction`; engineering.md "IMSIN and IMCOS reduce
  the real part by the x87 64-bit pi".
- Candidates (kind (c), a structural model): libm (old), `excel_sin_cos` (chosen), and the 66-bit-pi reduction the task
  suggested. For x = 1.5707963267949 the remainder Excel implies (2.88915216855774E+293 / 8.27492013840132E+307 =
  3.491456256055E-15) is x - pi64/2 with the x87 64-bit pi rounded to nearest (3.4914562560550699E-15); the true
  remainder is 3.49148134E-15 and a 66-bit pi gives 3.49148336E-15, so only the existing x87 kernel fits.
- No other cell changes: a full `rustsheet-eval run` with main's build and this one differs only in the t3616 case
  (3521 cases compared cell by cell). Fit labs `im-hyperbolic` 1359/1359, `im-hyperbolic-fresh` 1200/1200,
  `im-small-imag` 600/600, `im-hyperbolic-t3589-fresh` 599/600, `im-hyperbolic-log` 891/891 and
  `im-hyperbolic-log-fresh` 300/300 are identical for both builds.
- Decision 0188 R4 (check 7, kind (c), no `FITTED` tag): hold-out is the fit lab's split (sha256
  'rustsheet-fitlab-holdout/v1', 30%) with its margin (test exact rate at most 5 points below train), both fixed before
  scoring. New fit `im-near-half-pi` (the fresh sweep and the t3616 probe, 664 IM* texts): train / test exact
  chosen 469/469 / 195/195 (100% / 100%), correctly rounded 35/469 / 13/195, main's libm build 199/469 / 89/195; accept.
- Fresh sweep (R4.2): probes-numeric/t3726-imsin-imcos-near-half-pi-sweep, generated after ecc720f39 froze the change
  (docs/research/t3726/gen_sweep.py, seed 3726; 200 rows, x = k*pi/2 + d to 15 digits, k = +-1..+-4, d = 0 on every
  third row, else |d| log-uniform in [1E-14, 1E-3] with a random sign; |y| log-uniform in [1, 709] with a random sign;
  A IMSIN, B IMCOS; excel-graph 2026-11-01). By text: 400/400 with this change, 28/400 with libm.
- Gate A envelope (R2): all 400 texts equal Excel's 15 digits, so all are inside; none outside.
  docs/research/t3726/excel_vs_math.py against 50-digit mpmath: 226 of 800 parts are more than 1e-9 off the math and
  36 more than G = 1e-4 (all at x = +-4.71238898038469, the 15-digit neighbour of 3pi/2, 4.1E-4, the same factor as
  `COS(RADIANS(90))`'s 6.1257422745431E-17, numbers.md). Decision 0188 R3 lists no SIN/COS/IM* case and only an owner
  ruling adds one, so Excel's value is the target; whether this becomes an R3 case is the owner's call.
- Reproducibility floor: the change is deterministic (the double-double reduction of `excel_sin_cos`); every sweep
  text matches bit for bit at 15 digits, no tolerance. No keep-matched branch: one path for every complex x below 2^27.
- Eval: probes-numeric +4 cells (ungated suite; the new sweep adds no baseline lines).
