---
id: T-3188
title: "Probe TEXT of huge positive numbers and tagged calendar prefixes"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(1E15,"b1")` and tagged prefixes over a value past the maximum serial match Excel.

## Context
T-3050 lets a literals-only calendar-prefix section through for any positive value past the maximum serial, but only 3000000 under bare `b1`/`b2`/`B1` was probed (probes/t3050-b1-positive-large). Code: `format_number` in formatter/format.rs.

## Acceptance criteria
- [ ] Probe `TEXT(1E15,"b1")`, `TEXT(3000000,"[$-40C]B1")` and `TEXT(3000000,"[$-40D]B1 x")` (excel-graph); record in number-formats.md; make our output match, baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed (excel-graph) `TEXT(1E15,"b1")`, `[$-40C]B1` and `[$-40D]B1 x` over 3000000 plus three companion cells: all literal-only, matching the T-3050 rule. Our output already matched, so no code change; findings recorded in number-formats.md. Eval delta: +6 passing cells (new case probes/t3188-b1-huge-tagged), no regressions.
