---
id: T-2140
title: "Gate the BESSEL asymptotic model: m9-bessel subset in probes, plus x above 100"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The `|x| >= 8` BESSELJ/BESSELY model from T-0908 is protected by a gated suite, and Excel's behavior above x = 100 is known.

## Context
T-0908's fidelity review: the 362 m9-bessel cells that model makes bit-exact sit in probes-numeric, which is not gated (`gate = false`). Only BESSEL.xlsx (x = +-10, +-32) guards the branch. The model is also only evidenced for 8 <= x <= 100. Excel's own SIN changes behavior near 2^27, so very large x may differ. Code: `functions/engineering/transcendental/bessel_excel.rs` (`asymptotic`).

## Acceptance criteria
- [ ] A probes case (tag M9) re-asks Excel for a subset of m9-bessel: x = 8, 8.5, 9, 12.5, 64, 100 at orders 0, 1, 4, 8, plus `BESSELJ(8.5,4)` and `BESSELY(8.5,1)`. It is oracle-filled and passing.
- [ ] The same case covers x in {1e3, 1e6, 134217728, 1e10} for both functions at orders 0, 1, 3. engineering.md records the result, and any mismatch is filed or fixed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The order cap (T-2139).
