---
id: T-0061
title: "Fifteen-digit closeness bucket in the eval report and parity page, with a ratchet on visibly-off cells"
milestone: M0
lane: harness
status: in-progress
priority: P1
size: M
tier: opus
campaign: visible
depends_on: []
---

## Goal
The eval report and the parity page show how many cells and functions are visibly off from
Excel: numbers more than half a unit in the 15th significant digit away, and wrong types, errors
or text. A gate keeps that count from growing. The visible-parity campaign (decision 0083) needs
this to measure progress and to say when it is done.

## Context
- `crates/rustsheet-eval/src/closeness.rs` (T-2036) has five buckets: pass, ulp1, ulp4, close
  (within 1e-9 relative) and off.
  - "close" hides numbers right to only 10 digits.
  - "off" mixes numbers with 2 correct digits and cells of the wrong type or text.
- `tools/src/rustsheet_tools/parity_report.py` renders these buckets on docs/parity.md (decision
  0079).
- Measured by hand on the 2026-10-20 run (scripts in the T-0060 session):
  - visibly-off numeric cells: 4,613 (13–14 digits right 2,458; 10–12 digits 962; 6–9 digits
    516; fewer than 6, or 0 against nonzero, 677);
  - non-numeric mismatches: 1,459;
  - cells within half a unit that still print differently at 15 digits: 612. They sit on a
    rounding boundary, so they count as visible parity but not ulp parity.
  - Attribution: blame the specialised function in a formula, not glue such as IF, SUM, DATE or
    INDEX. Otherwise IF and SUM look visibly off because of what they wrap.

## Acceptance criteria
- [ ] `Closeness` splits the current `close` and `off` buckets:
  - `digits15`: within half a unit in the 15th significant digit of the expected value, more than
    4 ulp apart;
  - `visible_numeric`: numbers further than that, and 0 against nonzero;
  - `visible_other`: wrong type, error, text or boolean.

  The definitions are documented, with unit tests on boundary values: exactly half a unit, an
  expected 0, and subnormals.
- [ ] report.json carries the new counts per function and per suite, and report.md lists the
      functions with visibly-off cells, worst first.
- [ ] The parity page shows a "Visibly off" column for functions and cells. The landing summary
      states how many in-scope functions are visibly off.
- [ ] A ratchet file (for example `quality/visible-parity.json`, protected; decision record)
      holds the in-scope visibly-off cell count. `make check` fails when the count grows. The
      ratchet updates through `make eval-baseline`.
- [ ] The run of this PR's `main` reproduces the hand counts above within the attribution
      difference, explained in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing any function.
- Changing which cells pass or fail the existing gate.

## Visible-parity stage
This task is the campaign's measuring stick. After it lands, run the gap-triager so it files the
remaining visibly-off functions that no tagged task covers yet. Examples: WEIBULL.DIST,
NORM.S.DIST tails, LOGNORM.DIST, F.DIST, ODDFPRICE, AMORDEGRC, LINEST, LOGEST, GROWTH and the
IM* functions.
