---
id: T-3564
title: "BETA.DIST density at min < 1 < 8 < max: the 4,164 bit-level t3383-r4-algdiv cells within 15 digits"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3563]
defer: phase-1b
---

## Goal
`BETA.DIST(x,a,b,FALSE)` with min(a, b) < 1 < 8 < max(a, b) matches Excel to the bit, or within 1 ulp stated per
shape, on the two T-3383 probes. These cells are already within 15 digits but not bit-exact, and no task owns them.

## Context
- Gap triage 2026-11-01 (`make eval` on b98b3c00d). Of the 8,000 rows the two probes score, 2,304 fail in the hold-out
  case and 1,992 in the in-sample case. T-3563 owns the 132 that are visibly off (it lists them by cell). This task
  owns the other 4,164: 2,173 of the hold-out case and 1,991 of the in-sample case. The cases, whole:
  - probes-numeric/t3383-r4-algdiv-hold.case.json (hold-out: a = 0.75 and 0.4, b on [8.6, 7000])
  - probes-numeric/t3383-r4-algdiv-a.case.json (in-sample: a = 0.5 and 0.25, b on [8.25, 5000])
- Root cause: the same as T-3563's. The density's lnB = fl(GL(min) + algdiv(min, max)) is Excel's correctly rounded
  lnB plus -4 to +3 ulps of noise with no structure in b. TOMS algdiv in doubles reaches 2,008 and 1,696 of 4,000 rows
  (docs/research/t2464/r4-algdiv/RESULTS.md; statistics.md "Beta density for min(a,b) <= 1"). T-3468 item 2 is the
  research. T-3563 ports the model for the visibly-off cells, and this task measures what is left after it.
- The pins' offsets sit inside the 4-ulp web/desktop floor (docs/research/oracle-calibration.md). So this task may
  end with a recorded reproducibility limit rather than bit parity (decision 0188 R4.4).
- Excel GAMMALN at the probes' arguments: probes-numeric/t3383-r4-algdiv-gl.case.json (1): Calc!B4, one ulp off. It
  is listed here because it is one of the pieces of lnB.

## Acceptance criteria
- [ ] After T-3563, the bit-exact, within-1-ulp and within-4-ulp counts per case and per shape are in the Outcome,
      against T-3563's and TOMS-d's. Any further model of Excel's algdiv carries the decision 0188 R4 guards
      (reviewer-excel-fidelity check 7): hold-out counts for the chosen and the correctly rounded candidate, a fresh
      sweep after the freeze, the Gate A envelope result, a `FITTED(T-3564, n_cells, range)` tag and range, nothing
      fitted below the reproducibility floor, and no keep-matched branch.
- [ ] No cell of any suite goes from within 15 digits to visibly off; no gated bit-exact cell is lost without a
      decision 0082 record.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The visibly-off cells (T-3563); the mirror region b < 1 < 8 < a (T-3528).

## Deferred (decision 0114)
`defer: phase-1b`: every cell this task owns is already within 15 significant digits of Excel, so it waits until
Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells
visibly off (then give it a `campaign:` stage instead).
