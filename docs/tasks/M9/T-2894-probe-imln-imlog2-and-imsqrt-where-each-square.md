---
id: T-2894
title: "Probe IMLN, IMLOG2 and IMSQRT where each square is subnormal but their sum is normal"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's answer is evidenced for IMLN, IMLOG10, IMLOG2 and IMSQRT where both parts are nonzero, each square is
subnormal and their sum is normal, and the engine matches it.

## Context
- T-2572 fitted "a subnormal square is 0" (`flushed_square`, functions/engineering/complex/mod.rs) from
  probes/eng-imln-modulus-range A51-A56, A79, A80. Every probed point has either one square normal or a subnormal
  sum, so the band where both squares are subnormal (parts of about 1.06E-154 to 1.49E-154) but their sum is
  normal is extrapolated: ours is `#NUM!` for the logs and "0" for IMSQRT; before T-2572 they answered
  (reviewer-excel-fidelity on T-2572; docs/excel-semantics/engineering.md).
- Probe with the `excel-probe` skill: IMLN, IMLOG10, IMLOG2, IMSQRT and IMABS of `1.1E-154+1.1E-154i`,
  `1.2E-154+1.2E-154i`, `1.4E-154+1.4E-154i`, `1.2E-154+1.4E-154i`, and one part 1.2E-154 with the other 1E-153.

## Acceptance criteria
- [ ] A probe case with excel-graph values for the cells above is committed.
- [ ] The engine matches every cell, or the rule in engineering.md is corrected and the mismatch is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The overflow side (squares past DBL_MAX).
