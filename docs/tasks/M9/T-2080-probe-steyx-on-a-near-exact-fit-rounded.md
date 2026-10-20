---
id: T-2080
title: "Probe STEYX on a near-exact fit whose residual sum rounds below zero"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Know what Excel's STEYX returns when its residual sum `Syy - Sxy*Sxy/Sxx` rounds to a tiny negative number on a
(near-)exact linear fit, and match it.

## Context
T-1723 made STEYX `sqrt((Syy - Sxy*Sxy/Sxx)/(n-2))` over the sums about the means, which fits every probed cell to the bit
(probes/regression-pairs-b M column). On an exact fit with non-integer data the radicand can round below zero; ours then
returns `#DIV/0!` (non-finite result), Excel may return 0, `#NUM!` or something else: unprobed. Write a probe case
(excel-probe skill) with exact fits on non-integer data (e.g. y = 0.1*x + 0.3 over x = 1..7, y = 3x on x = 0.1, 0.2, 0.3)
and compute the radicand for each in a scratch script to pick ones that go negative; then match Excel and record it in
docs/excel-semantics/statistics.md "Paired regression functions".

## Acceptance criteria
- [ ] A probe case with oracle values for at least three exact fits whose radicand rounds negative, and three that round to +tiny.
- [ ] STEYX matches those cells; the finding is recorded in docs/excel-semantics/statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- LINEST's standard error (T-1741).
