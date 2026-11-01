---
id: T-3772
title: "BETA.DIST density at shapes c+j against a0 > 15: owner for the 7,026 last-digit t3568 terms cells (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
The BETA.DIST density cells of the two t3568 terms probes are bit-exact with Excel. Alternatively, a cell can move to
a narrower open task that names its density branch, or be listed in the Outcome with Excel, ours, the 50-digit math,
and the reason Excel's bits cannot be derived.

## Context
- Gap triage 2026-11-01 (`make eval` on origin/main, `make unowned-off ALL=1`): these are the only visibly-off Excel
  cells that no open task names, deferred tasks included. Every other unowned group has an owner already, or one is
  being filed on an open branch (T-3755, T-3757, T-3764).
- The probes come from T-3568 (docs/research/t2464/r5-bup/RESULTS.md section 4, `mkterms.py`). At 300 BETA.DIST points
  of each half, each cell is `BETA.DIST(y0, c + j, a0, FALSE)` for every j < n. Here c is in (0, 1], j goes up to 39,
  a0 > 15 and the second shape is up to about 3,000. They tested whether bup's sum is a loop of one-term bups on
  Excel's own densities. It is not (negatives table: 5/30/80 of 300). So no task took the cells themselves.
- Cases (`make eval-show CASE=<case>`), all cells of each:
  - probes-numeric/t3568-r5-bup-terms-tune.case.json: 5,898 checked, 275 bit-exact, 3,602 visibly off
  - probes-numeric/t3568-r5-bup-terms-hold.case.json: 5,967 checked, 283 bit-exact, 3,424 visibly off
- Examples (hold):
  - `BETA.DIST((20564950*2^-27+24281061*2^-53),95701492081623*2^-47,1769.6,FALSE)`: Excel 3.975680312655546e-126,
    ours 3.9756803126555495e-126 (NumericClose)
  - `BETA.DIST((38012339*2^-27+41459319*2^-53),(5842497*2^-24+46965467*2^-50),20.3,FALSE)`: Excel
    0.0040816465992292335, ours 0.004081646599229234 (NumericUlp)
- Why this is Phase 1b (decision 0188 R1(c) and R5). A throwaway mpmath check (50 digits, density from
  exp((a-1)ln x + (b-1)log1p(-x) - ln B(a, b))) covered all 7,026 visibly-off cells. Excel and ours are both within
  1e-9 relative of the math on 7,026 of 7,026. Excel is within half a unit in the 15th digit of the math on 2,403 and
  ours on 3,038, and ours is the closer of the two on 4,556. Excel is the less accurate side, so this is last-digit
  work on the BETA.DIST density, which R5 parks. By the smaller shape: 20 below 1, 2,494 in [1, 8) and 4,512 at 8 or
  more. The second shape is below 100 on 271 cells, 100 to 1,000 on 4,090 and 1,000 or more on 2,665.
- Related density work (keep the lists disjoint; move cells there if a branch matches):
  - T-3392: the Loader density at the bit for min(a, b) > 1.
  - T-3244: both shapes large, a + b - 2 >= 90.
  - T-3220: min(a, b) > 1, the Loader constant and bd0 residual.
  - T-3569 / T-3750: algdiv's lnB. On its branch, T-3569 is negative on web pins and T-3750 waits for desktop.
- Code: crates/rustsheet-engine/src/functions/statistical/beta_density.rs, large_shape_beta.rs, loader.rs.

## Acceptance criteria
- [ ] Every cell of the two cases above is one of these:
      - bit-exact;
      - moved to a named narrower open task, and listed there;
      - listed in the Outcome with Excel, ours, the 50-digit math and the reason.

      probes-numeric is ungated (no baseline file), so the before and after counts come from the report. No baseline
      under eval/baseline/ gains a line.
- [ ] Any kernel change carries decision 0188's R4 guards (reviewer-excel-fidelity check 7):
      - hold-out counts for the chosen and the correctly rounded candidate, on a split fixed before scoring;
      - a fresh log-uniform sweep drawn after the freeze, with its counts;
      - the Gate A envelope result on that sweep;
      - a `FITTED(T-3772, n_cells, range)` tag and range on fitted constants, with accurate math outside the range;
      - nothing fitted below the oracle's reproducibility floor;
      - no keep-matched branch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The t3568-r5-bup tune, hold and fresh cases (columns A, B and C), which T-3712, T-3764 and T-3150 own.
- Visibly-off BETA.DIST cells outside these two cases.

`defer: phase-1b`: every cell here is R1(c) Phase 1b evidence. Excel and ours are both within 1e-9 of the 50-digit
density, and R5 parks last-digit work on the BETA.DIST density.
