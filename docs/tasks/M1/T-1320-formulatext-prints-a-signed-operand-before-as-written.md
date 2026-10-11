---
id: T-1320
title: "FORMULATEXT prints a signed @ operand before ^ as written: =@-2^2 (engine prints =(@-2)^2; T-1285)"
milestone: M1
lane: engine
status: done
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
- [x] `probes/at-signed-operand.case.json` I61 passes and leaves the baseline.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed 12 `@`/`^` forms (probes/at-power-text, excel-graph): Excel never parenthesises `@` as a power operand and keeps the written parentheses of `(@-2)^2`. Engine: `@` is now a bare power operand in `operator_text` (stringify.rs); `=@-2^2` and 9 other forms match. Two cells stay baselined: I26 `=@A1^2^3` (T-1331) and I46 `=(@-2)^2` (the AST does not keep the written group; T-1244/T-1116). Eval delta: probes known failures 939 -> 940 (fixed 1, new-coverage failures 2), 0 regressions.
