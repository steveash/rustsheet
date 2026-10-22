---
id: T-2389
title: "Probe IMLN and IMLOG10 of 0 (may leak -inf text)"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IMLN("0") and IMLOG10("0") answer what Excel answers (probably `#NUM!`), not a `-inf` text leak.

## Context
Found reviewing T-2334: imln/imlog10 in functions/engineering/complex.rs share the `r.ln()` path that leaked `-inf` for IMLOG2("0"). No oracle cell covers them; probe with the `excel-probe` skill first.

## Acceptance criteria
- [ ] Oracle values for IMLN("0") and IMLOG10("0") recorded; engine matches them, with a unit test.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
