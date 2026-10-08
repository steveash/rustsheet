---
id: T-1186
title: "ERFC(26.4) is a few ulp off Excel in the erfc continued-fraction tail"
milestone: M1
lane: functions-math
status: blocked
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
ERFC and ERFC.PRECISE match Excel bit-exactly in the far tail (x of about 26).

## Context
probes/subnormal-function-boundaries cell A5: `ERFC(26.4)` is 4.4017768588035533E-305 in Excel (excel-graph, T-1180), the engine gives 4.401776858803543E-305 (about 2E-15 relative). `erfc_cf` in crates/rustsheet-engine/src/functions/statistical/special.rs is the tail path. Numeric fitting against Excel's cached values, hence tier opus.

## Acceptance criteria
- [ ] Cell A5 of probes/subnormal-function-boundaries passes, with no regression in ERFC cells elsewhere (`make eval-baseline` records it).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the subnormal flush (T-1180).

## Blocked because
Excel's ERFC core is not identified, so A5 cannot be matched on evidence; it is only possible by overfitting one point.
- **What is fixed:** the branch drops the engine's compensation for the rounding of x*x in `erfc_cf`. Excel visibly uses `exp(-fl(x*x))`: 2.27E-15 of A5's 2.40E-15 offset from the true erfc is exactly that rounding. A5 moves from about 21 ulp off to 1 ulp off, with 0 regressions in every gated suite. On the 1/8 grid x*x is exact, so nothing there changes.
- **What remains:** the core factor erfc(x)·exp(x²) is up to ±5 ulp from correctly rounded on the probes-numeric/t1040-erfc grid, and it is not identified. None of these reproduces more than 22% of the 206 grid points with x >= 1: the bottom-up CF (7 product/division orders), Cody CALERF (with and without its x*16 split), and the asymptotic series (5 orders, either summation direction). One reversed-sum asymptotic form happens to hit A5, but it matches the grid no better than the CF (32/150 at x >= 8), so adopting it would fit one cell.
- **Next:** a research task to identify Excel's erfc core. Candidates: Excel's NORM.S.DIST tail clustering at 2^-43 (statistics.md), a W. J. Cody/Hart rational with a different split, or an incomplete-gamma CF. Score each candidate on the full t1040 grid, not on A5. Findings are in docs/excel-semantics/statistics.md (ERFC sweep).
