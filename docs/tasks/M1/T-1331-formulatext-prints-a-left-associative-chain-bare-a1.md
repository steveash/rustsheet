---
id: T-1331
title: "FORMULATEXT prints a left-associative ^ chain bare: =@A1^2^3 and =A1^2^3 (engine wraps =(@A1^2)^3; T-1320)"
milestone: M1
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
FORMULATEXT of a left-associative `^` chain prints without added parentheses, as Excel does.

## Context
Excel prints `=@A1^2^3` (value 729, left-associative) as written; the engine prints `=(@A1^2)^3` because the power arm of `operator_text` (parser/stringify.rs) wraps a power-chain left operand. Failing cell probes/at-power-text.case.json Sheet1!I26 (T-1320). Check plain `=A1^2^3` with a probe first.

## Acceptance criteria
- [ ] `probes/at-power-text.case.json` I26 passes and leaves the baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
