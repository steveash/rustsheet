---
id: T-1111
title: "Probe : right-operand errors, ANCHORARRAY of a non-reference and @-operand parentheses"
milestone: M1
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Oracle values for the forms T-1109 left UNVERIFIED, so the engine can be made to match.

## Context
T-1109 made `:` return its left operand's error and kept `#VALUE!` for a right-operand error without evidence. Probe: `=A1:(1/0)`, `=A1:INDEX(A1:A3,5)`, `=(1/0):(A1 B1)`, `=_xlfn.ANCHORARRAY((@A1:B2))`, `=ANCHORARRAY(1)` (value and FORMULATEXT), and FORMULATEXT of `=@((A1 B1))`, `=@(A1:B1)`, `=@(A1)`. Also `=A1:(1/0)` prints `=A1:1/0` in the engine, which does not re-parse to the same formula.

## Acceptance criteria
- [ ] A probe case in eval/suites/probes with oracle values for each form above; findings in docs/excel-semantics/references.md (replace the UNVERIFIED notes); implementation tasks filed for any mismatch.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
