---
id: T-3803
title: "BETA.DIST density on the t3568-r5-bup-terms probes: owners for the 11,307 failing density cells (Phase 1b, both within 1.7e-13 of the math)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2586]
defer: phase-1b
---

## Goal
Every failing cell of probes-numeric/t3568-r5-bup-terms-tune and probes-numeric/t3568-r5-bup-terms-hold, all of them
`BETA.DIST(x,a,b,FALSE)`, is within half a unit of the 15th significant digit of Excel's value, or is moved to a
narrower density task that names its branch (both files name each other), or is listed in the Outcome with Excel,
ours and the 50-digit math and the reason Excel's digits cannot be derived.

## Context
- Gap triage 2026-11-02 (`make eval` on origin/main 56917e82c, excel-graph). `make unowned-off ALL=1` and
  `make exit-owners MILESTONE=M9` list these two cases as the only BETA.DIST cells with no open owner: 7,026 visibly
  off (all in the M9 rate criterion as `rate:BETA.DIST`) and 4,281 more within 15 digits.
- T-3568 (done) made the probes for its bup-loop hypothesis (docs/research/t2464/r5-bup/RESULTS.md, `mkterms.py`):
  each cell is Excel's density `BETA.DIST(y0, c + j, a0, FALSE)` at a bup point's shapes. They were evidence for the
  bup+bgrat fit, not cells that fit fixes, so neither T-3568 nor its follow-ups (T-3712, T-3719) name them. T-3712 owns
  the bup cells of t3568-r5-bup-tune, -hold and -fresh, not these.
- The triage computed the 50-digit density (mpmath, `x^(a-1)(1-x)^(b-1)/B(a,b)` on the double inputs) for the 7,026
  visibly-off cells. Both Excel and ours are within 1.7e-13 relative of it on every cell. Within 1e-14: Excel on
  3,905 and ours on 4,593. Excel and ours are both within 1e-14 on 3,356. Decision 0188 R1(c) makes this Phase 1b
  evidence, hence `defer: phase-1b`, as for T-3244 and T-3592. They count in the R1(c) rows today only because
  BETA.DIST has no Gate A definition yet (T-3529 adds it). If Gate A's tables show a cell outside the envelope, file
  that cell as its own task without the key.
- Shapes (x from 0.0093 to 0.2993). Each band has an owner task for the density kernel:
  - **min(a, b) > 1 and a + b - 2 >= 90 (10,228 cells: hold 5,164, tune 5,064).** a is 1.0085 to 39, b is 70.4 to
    3,000. This is the Loader saddle point of `loader.rs` `excel_beta_density`. T-3244 (n = 90 to 256), T-3392 (bit
    level) and T-3220 (the port's newly-off cells) are its owners, all waiting on T-2586's model of the Loader
    constant and bd0 series. Most of these cells have n above 256, past T-3244's range.
  - **min(a, b) > 1 and a + b - 2 < 90 (749 cells: hold 353, tune 396).** a is 1.03 to 23.5, b is 17.3 to 90.8.
    Same kernel.
  - **min(a, b) <= 1 (330 cells: hold 167, tune 163).** a is 0.01 to 0.993, b is 17.3 to 3,000.
    `beta_density.rs` `beta_density_small_shape`, owned by T-3035 (visible), T-2821 (bit level) and T-3563/T-3564
    (min < 1 < 8 < max, algdiv).
- Cases owned, whole (decision 0136: each case is named on a line with no cell):
  - probes-numeric/t3568-r5-bup-terms-tune
  - probes-numeric/t3568-r5-bup-terms-hold
- probes-numeric is not gated, so no eval/baseline line moves. Measure with `make eval-show CASE=probes-numeric/t3568-r5-bup-terms-hold.case.json` and the report's per-case counts.

## Acceptance criteria
- [ ] Each failing cell of the two cases is within half a unit of Excel's 15th significant digit, or moved to a named
      narrower task (both files name each other), or listed in the Outcome with the three values and the reason.
- [ ] Any fitted change meets the R4 guards of reviewer-excel-fidelity check 7 (decision 0188):
      - hold-out counts stated before scoring (the -hold case is the hold-out of the -tune case, as T-3568 split them);
      - a fresh sweep after the freeze;
      - the Gate A envelope result;
      - the `FITTED` tag with its range;
      - the reproducibility floor;
      - no keep-matched branch.
- [ ] No gated known-failure line is added (`make eval-baseline` refuses regressions), and the fit sets of T-3244,
      T-3220 and T-3035 keep their counts.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cdf cells of t3568-r5-bup-tune, -hold and -fresh (T-3712), and the a0 < 1e-15 band (T-3719).
- The Loader constant and bd0 series themselves (T-2586): this task ports them, it does not crack them.
