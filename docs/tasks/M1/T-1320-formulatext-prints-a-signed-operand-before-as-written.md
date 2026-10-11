---
id: T-1320
title: "FORMULATEXT prints a signed @ operand before ^ as written: =@-2^2 (engine prints =(@-2)^2; T-1285)"
milestone: M1
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
FORMULATEXT of `=@-2^2` prints `=@-2^2` as Excel does.

## Context
Excel value is 4 and text `=@-2^2`; the engine's value is right but it prints `=(@-2)^2`. Failing cell probes/at-signed-operand.case.json Sheet1!I61 (T-1285). docs/excel-semantics/references.md.

## Acceptance criteria
- [ ] `probes/at-signed-operand.case.json` I61 passes and leaves the baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
