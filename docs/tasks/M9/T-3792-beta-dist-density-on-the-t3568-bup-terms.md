---
id: T-3792
title: "BETA.DIST density on the t3568 bup-terms probes: Loader residue at a + b - 2 up to 3,000 (7,026 unowned visibly-off probes-numeric cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3244, T-3250, T-3220, T-3035]
campaign: visible
defer: phase-1b
---

## Goal
`BETA.DIST(x,a,b,FALSE)` on the two t3568 bup-terms probes is within 15 digits of Excel's value on the 7,026
visibly-off cells, or each cell left over is named with its reason and an owner. The density models that T-3244,
T-3250, T-3220 and T-3035 fit are checked on this independent data before they are trusted, with no new
visibly-off cell anywhere.

## Context
- Decision 0188 R5 (owner's 2026-10-31 rulings): parked as Phase 1b. Every cell this task owns is a BETA.DIST
  density cell of a dense probes-numeric sweep inside Gate A's envelope (measured below), so R1(c) counts it as
  Phase 1b evidence. A point that Gate A's tables show outside the envelope gets its own task without the key.
- Gap triage 2026-11-01 (`make eval` on origin/main fafb6c6f, excel-graph; `python3 scripts/exit_owners.py
  --all-off`, also run against the task files of the unmerged branches agent/functions/tasks-gap-triage-opus-2026-11-01n
  and agent/research/tasks-plan-opus-m2-2026-11-01). These were the only non-deferred unowned visibly-off cells of
  the run: 3,602 in t3568-r5-bup-terms-tune and 3,424 in -terms-hold. Both count in M9's `rate:BETA.DIST`.
- T-3568 (done) wrote the probes to test its "loop of one-term bups" hypothesis: `BETA.DIST(y0, c + j, a0, FALSE)`
  for every j < n at 300 BETA.DIST points of each half (docs/research/t2464/r5-bup/RESULTS.md, `mkterms.py`). The
  hypothesis was rejected, so nothing owned the cells afterwards. T-3712 owns only the bup-tune, -hold and -fresh
  cases. The cells are plain densities: one shape is c + j in (0, 40), the other is a0 from 15 to 3,000.
- Breakdown (mpmath at 40 digits on the doubles the formulas give; |error| in units of 2^-52 relative, median /
  p90, for the visibly-off cells):

  | region | cells | visibly off | Excel | ours | domain owner |
  |---|---|---|---|---|---|
  | min(a,b) > 1, n = a + b - 2 > 256 | 7,234 | 5,686 | 50 / 243 | 36 / 168 | T-3250 (large exponent) |
  | min(a,b) > 1, 90 <= n <= 256 | 2,994 | 1,183 | 15 / 40 | 9 / 25 | T-3244 |
  | min(a,b) > 1, n < 90 | 749 | 137 | 9 / 21 | 6 / 13 | T-3220 |
  | min(a,b) <= 1 | 330 | 20 | 42 / 216 | 35 / 220 | T-3035 |

  The largest relative error of either side is 2.7e-13, so every cell is inside Gate A's 1e-9 envelope. In each
  region Excel is further from the math than we are, so the stage is `visible`, not `accuracy`. 215 cells have
  Excel within 2.5 ulp of the math while we are more than 8 ulp off, but 326 cells show the reverse. That is the
  scatter of two noisy evaluations, not a region where Excel is accurate.
- Examples (Excel vs ours):
  - terms-tune Calc!N233 `=BETA.DIST((13860488*2^-27+28965031*2^-53),(28521267*2^-21+13421773*2^-47),2810.0,FALSE)`:
    1.4977104323629519e-108 vs 1.4977104323629131e-108 (Excel +122 ulp from the math, ours +5)
  - terms-hold Calc!C7 `=BETA.DIST(0.214001149,(8598323*2^-22+13421773*2^-48),195.0,FALSE)`: 4.9684636464866776e-17
    vs 4.9684636464866924e-17
  - terms-hold Calc!K266 `=BETA.DIST((35254923*2^-27+35854565*2^-53),10.1602,54.1868,FALSE)`: 0.7858307640005299
    vs 0.7858307640005293
  - terms-hold Calc!A257 `=BETA.DIST((30135901*2^-27+6652561*2^-53),(1887436*2^-21+53687091*2^-47),183.0,FALSE)`:
    9.39678537483394e-19 vs 9.39678537483393e-19 (min shape 0.9)
- Why a separate task: the fixes belong to the domain owners above, so this task depends on them. It owns the
  cells so they are not unowned, and it is where those models are checked on data none of them was fitted on.
  The probes span n from about 15 to 3,000, wider than T-3244's 90-256 and T-3250's single a = 1025 column plus
  T-3249's sweep. Use these cases only as a hold-out, never to tune (decision 0056, R4.1).
- Semantics: docs/excel-semantics/statistics.md "Beta density for min(a,b) > 1 is Loader's saddle point" and "Beta
  density for min(a,b) <= 1: lnB and a double-double exponent split". Code:
  `crates/rustsheet-engine/src/functions/statistical/loader.rs` (`excel_beta_density`, `excel_bd0`),
  `beta_density.rs`.

Owned cases (BETA.DIST density only; the task owns every visibly-off cell of both):
- probes-numeric/t3568-r5-bup-terms-tune.case.json (3,602)
- probes-numeric/t3568-r5-bup-terms-hold.case.json (3,424)

## Acceptance criteria
- [ ] Once the four dependencies have landed, the Outcome gives each region's count of visibly-off cells on the two
      cases (the table above, re-measured), with exact, 4-ulp and 15-digit counts per region.
- [ ] Every cell still visibly off is fixed here, or named with its reason and an owner in a follow-up task.
- [ ] Any model change made here is a numeric fit and carries reviewer-excel-fidelity check 7's R4 guards
      (decision 0188), listed in the Outcome:
  - hold-out counts (train and hold-out side by side: exact, 4 ulp, 15 digits; the correctly rounded candidate on
    the same rows), with the split fixed before scoring;
  - a fresh log-uniform sweep, oracle-filled after the freeze, with its Gate A envelope result listing every
    point outside;
  - a `FITTED(T-3792, <n> cells, <range>)` tag on each fitted constant or cut, applied only in that range, with
    accurate math outside it;
  - no fitting below the oracle's reproducibility floor (4 ulp, docs/research/oracle-calibration.md);
  - no branch that only exists to keep cells that already match.
- [ ] No new visibly-off cell in any suite, and the eval/baseline known-failure lines only shrink. probes-numeric
      has no baseline file, so the measure is `make unowned-off ALL=1 FUNCTION=BETA.DIST` (and the report's
      `attributed_off`) for the two cases, before and after, in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The bup+bgrat cdf composition (T-3712) and the bup-tune, -hold and -fresh cases.
- Changing the density models themselves before their owners land (T-3244, T-3250, T-3220, T-3035).

## Visible-parity stage
Target (decision 0114): every cell above within half a unit of the 15th significant digit of Excel's value.
Bit-level residue goes to a follow-up with `defer: phase-1b`.
