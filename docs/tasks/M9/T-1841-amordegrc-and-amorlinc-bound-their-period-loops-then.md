---
id: T-1841
title: "AMORDEGRC and AMORLINC bound their period loops, then lift over arrays (SPEC-0011)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: S
tier: opus
depends_on: [T-1822]
---

## Goal
AMORDEGRC and AMORLINC lift over arrays in every argument and refuse a multi-cell range there, without a call able to loop for an unbounded time.

## Context
Split from T-1822. `amor_linc` and `amor_degrc` (functions/financial/financial_depreciation.rs) loop `while counted <= period`,
and `period` is only bounded by `ceil(1/rate)`: a tiny rate and a period of 1E15 loop 10^15 times today (directive 5).
Excel gives 0 for `AMORLINC(2400,39679,39813,300,1E9,0.15,1)` and `AMORDEGRC(...)` (probes/fin-remaining-later-argument-ranges
K4, K5), where the period is past the asset life. Bound the loops (the depreciation is constant or 0 once the
available amount is used up, so the loop can stop early; or a cap with a DEVIATION entry), then enter both as
`LiftArrays`: every argument is probed over an array (fin-argument-modes H19-H35) and over a range (H20, H29;
fin-remaining-later-argument-ranges H11-H34, all `#VALUE!`).

## Acceptance criteria
- [ ] `AMORLINC(2400,39679,39813,300,1E15,1E-300,1)` finishes in milliseconds; any capped result is a DEVIATION.
- [ ] The AMORDEGRC and AMORLINC lift cells of probes/fin-argument-modes pass (baseline lines removed), no new failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
- AMOR* base values (probes/fin-amordegrc).
