---
id: T-3452
title: "BETA.INV at a = b = 1E-4 near p = 0.5 is visibly off because BETA.DIST's last digits differ there (BETADIST_BETAINV H26)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3451]
campaign: visible
---

## Goal
BETADIST_BETAINV H26 is within 15 significant digits of Excel: BETA.INV at tiny equal shapes lands where Excel's does.

## Context
Gap triage 2026-10-31e (`make eval` on main 8a14b8b). H26 is one of the ironcalc-calc visibly-off cells behind M9's
unmet exit row (48 against an allowance of 36). `scripts/exit_owners.py --all-off` shows it owned only by `defer:` tasks.
- ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx (1): Sheet1!H26
- H26 `BETA.INV(G26,0.0001,0.0001,0.15,1.2)`: Excel 0.2339999999999406, ours 0.23400000000265136 (1.1E-11 relative).
- Its input G26 `BETA.DIST(0.234,0.0001,0.0001,TRUE,0.15,1.2)` is within 15 digits but not bit-exact: Excel
  0.4998779049300665, ours 0.49987790493006656; mpmath (50 digits, x = 0.084/1.05) 0.49987790493006460733. Excel is
  not correctly rounded here, so this is the `visible` stage (reproduce Excel), not `accuracy`.
- At a = b = 1E-4 the cdf is nearly flat around 0.5, so our crossing of our own cdf at Excel's G26 value lands
  1E-11 away. Either our cdf must match Excel's to the bit near x = 0.08, or Excel's inverse is not its cdf's crossing
  there. Telling these apart is a numeric fit.
- **Needs an oracle sweep**: the cached workbooks have three x points at a = 1E-4 and no BETA.INV at tiny equal shapes.
  T-3451 provides the fit, hold-out and fresh-sweep cases; do not start before it is done.
- Rule: statistics.md "BETA.INV solves Excel's own CDF to the crossing (T-1745)". Code: beta.rs `beta_quantile`,
  inverse_beta.rs, bratio (the a0 <= 1 branches: bpser, T-3192; T-3220 density).

## Acceptance criteria
- [ ] H26 is within half a unit of the 15th significant digit of Excel's value, and its line leaves
      `eval/baseline/ironcalc-calc.txt` with no new known-failure line in any baseline.
- [ ] Decision 0188 R4 guards (reviewer-excel-fidelity check 7):
  - [ ] hold-out counts: the Outcome gives the fit and hold-out point counts from T-3451's cases, and the hold-out
        result (within 15 digits of Excel) for the fitted model, measured without refitting;
  - [ ] fresh sweep: T-3451's unread third case is scored once, after the fit is frozen, and reported;
  - [ ] Gate A envelope result (decision 0188 R2) on the fresh sweep against a 50-digit mpmath reference, per stratum:
        0 points outside both 15 digits of Excel and 1E-9 relative of the math, 0 error-against-number cells;
  - [ ] every fitted constant or branch carries a `FITTED` tag in the code with the input range it was fitted on;
  - [ ] the reproducibility floor: the fit is reproduced from a committed script and the cases alone;
  - [ ] no keep-matched branch: no code path that exists only to keep specific cells' bits.
- [ ] statistics.md records the model and its range.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETADIST_BETAINV G5 and H5 (T-3449).
- Bit-level residue of G26 and other cells within 15 digits (Phase 1b, T-3374).

## Visible-parity stage
Decisions 0083 and 0114 (`visible`, time-boxed by decision 0056): done when H26 is within half a unit of the 15th
significant digit of Excel's value. Bit-level residue moves to a follow-up with `defer: phase-1b`.
