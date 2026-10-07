---
id: T-1026
title: "INDEX and MATCH over an IF-wrapped 3-D span give #VALUE!, not #REF!"
milestone: M1
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`=INDEX(IF(TRUE,Data1:Data3!A1:A2),1)` and the same wrapped in MATCH give `#VALUE!` like Excel.

## Context
Found by T-1015. Failing: probes/3d-ref-functions Calc!C18 and C20 (engine `#REF!`, Excel `#VALUE!`); CHOOSE-wrapped C19/C21 already pass. See docs/excel-semantics/references.md.

## Acceptance criteria
- [ ] C18 and C20 pass and leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`evaluate_single_sheet_arg` now routes an IF argument through `if_impl` with `IfBranchMode::SingleSheet`, so a 3-D span
chosen by IF is `#VALUE!` in INDEX/MATCH like CHOOSE. Eval delta: probes/3d-ref-functions C18, C20 fixed (probes known
failures 193 -> 191), 0 regressions.
