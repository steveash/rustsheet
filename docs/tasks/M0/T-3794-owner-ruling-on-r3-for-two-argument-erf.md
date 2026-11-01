---
id: T-3794
title: "Owner ruling on R3 for two-argument ERF where the two values nearly cancel (T-3784 follow-up)"
milestone: M0
lane: harness
status: blocked
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Record the owner's ruling on whether Excel's two-argument `ERF(a,b)`, where erf(a) and erf(b) nearly cancel, is a
decision 0188 R3 (`MATH-OVER-EXCEL`) case. Until the owner rules, the engine keeps matching Excel.

## Blocked because
Blocked because: needs an owner ruling (decision 0188 R3: "no agent, steward included, adds a case on its own").

## Context
- `ERF(a,b)` is the exact difference of the two rounded single values (statistics.md, T-1778 entry, two-argument
  bullet). Where the two values nearly cancel, Excel's error relative to erf(b) - erf(a) is about
  2^-52 * max(|erf a|, |erf b|) / |erf(b) - erf(a)|, which can be far above R3's G = 1e-4.
- Evidence: probes-numeric/t1916-erf-wrappers (excel-graph). Its math sidecar landed with T-3784. 602 of its 915
  two-argument cells are more than 1e-4 relative off the 50-digit math, from 3.9E-4 to 3.28. For example:
  - D1530 `ERF(0.5000000000000002,0.5000000000000003)`: Excel -2.220446049250313E-16, math 9.756446527736046E-17,
    the opposite sign;
  - D1531: Excel 0, math 9.756E-17;
  - D1532: Excel 3.3306690738754696E-16.
- The engine matches Excel at these cells (directive 1). T-3784's Gate A table leaves the region out: a two-argument
  point is redrawn while |erf b - erf a| < 1e-9 * max(|erf a|, |erf b|). So the gate does not depend on this ruling.
- The owner's options:
  - (a) keep matching Excel, so this is no R3 case;
  - (b) an R3 case over a bounded region (the cancellation ratio, and which a and b), with a decision record.

## Acceptance criteria
- [ ] A decision record carrying the owner's ruling, or the owner's "keep matching Excel" written into the
      statistics.md two-argument ERF bullet.
- [ ] If (b): a follow-up implementation task with its region, and the region's evidence committed first
      (decision 0188 R3).

## Out of scope
- Changing the engine before the ruling.
