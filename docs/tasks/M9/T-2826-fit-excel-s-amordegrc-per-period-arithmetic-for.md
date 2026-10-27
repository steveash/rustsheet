---
id: T-2826
title: "Fit Excel's AMORDEGRC per-period arithmetic for charges of 1E12 and more (fin-amor-period-bound-c A1-A5, -b B2 to the bit)"
milestone: M9
lane: functions-financial
status: done
priority: P3
size: M
tier: opus
campaign: semantics
depends_on: [T-1895]
---

## Goal
AMORDEGRC's walk reproduces Excel's per-period arithmetic closely enough that probes/fin-amor-period-bound-c A1-A5 and
fin-amor-period-bound-b B2 have Excel's value, within the kernel's work cap.

## Context
From T-1895 (docs/excel-semantics/financial.md "AMORDEGRC past 2^20 periods"):
- Past 2^20 periods the kernel decays in closed form (a DEVIATION for the work cap); a full double walk of main's loop
  (charge unrounded, subtracted unrounded) gives Excel's -c A1, A3 and A4 but not A2 (...031, Excel ...030) nor A5.
- Subtracting each charge rounded to a whole number gives all of A1-A4, but breaks fin-amor-period-bound B4, B5, B11,
  B12, B14 and fin-amordegrc A7 (B4: 1000 where Excel is 0; B11: 41042641306981, Excel ...7062), so Excel does not round
  each charge before subtracting it (T-1895, a scratch run).
- A5 `AMORDEGRC(Z1,..,2000000,1E-7,1)` (Z1 = 1E300) is 1.51632679480347E293, a 15-digit value; every walk variant tried
  (main's, rounded subtraction, a 15-digit cut of each charge, accumulating the depreciation) gives ...348 at 15 digits.
  B2 (`AMORDEGRC(1E308,..,1E5,1E-6,1)`) is 1.94700443465919E302, ours 1.9470044346591894E302: Excel's result looks cut to
  15 significant digits at these sizes (ROUND's two-stage rounding, numbers.md, is a candidate for the final rounding).
- The double walk's rounding noise over 10^6 periods is about 1e-13 relative, so a fit needs Excel's exact operation
  order; probe smaller period counts (10, 100, 1000) at costs 1E20 and 1E300 to localise where Excel and main part.
A walk past 2^20 periods costs more work per element (about 4 ms per 2^20 periods): any cap change goes through the
work-cap entry ("AMORLINC and AMORDEGRC work cap") and `AMORDEGRC_WEIGHT`.

## Acceptance criteria
- [x] probes/fin-amor-period-bound-c A1-A5 and fin-amor-period-bound-b B2 pass (baseline lines removed), or the entry
  records why they cannot within the work cap, with no new failures.
- [x] `make check` green; reviewers approve.

## Out of scope
- AMORDEGRC below period 1 (T-2825).

## Outcome
AMORDEGRC (and AMORLINC's left-over value) now round a charge like ROUND(x,0): the exact value cut to 15 significant
digits, then half away (`amor_round` through `decimal_round::round_decimal`). Two new probes localise the walk
(probes/amordegrc-walk-ties, 69 cells, and probes/amordegrc-walk-drift, 66 cells, excel-graph 2026-10-27): the final
rounding is settled to the ulp at 1 and 2 periods, and Excel's walk drifts a few ulps from main's from about 1000
periods on, in both directions; ten candidate operation orders fit fewer cells than main's walk (financial.md "AMORDEGRC
rounds its charge like ROUND"). fin-amor-period-bound-b B2 passes; -c A2 passes as before; -c A1, A3, A4 would pass
with a walk past 2^20 periods but stay on the closed form within the work cap, and A5 needs Excel's walk (entry
"AMORDEGRC past 2^20 periods"). Eval delta: -1 known failure (B2), +135 probe cells of which 107 pass and 28 are
walk-drift known failures. Follow-up: T-2847 (Excel's per-period arithmetic).
