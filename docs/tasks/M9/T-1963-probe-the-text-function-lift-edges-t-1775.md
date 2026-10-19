---
id: T-1963
title: "Probe the text-function lift edges T-1775 left unverified: a LAMBDA argument, a 1x1 array, VALUETOTEXT's scalar error format"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-1775]
---

## Goal
docs/excel-semantics/text.md "Element-wise text functions on the lift driver" has Excel's values for the three edges T-1775 marked UNVERIFIED, and the engine matches them or a task is filed.

## Context
T-1775 moved LEFT, MID, FIND, REPT, TEXT, VALUETOTEXT, CONCATENATE and the other element-wise text functions onto the SPEC-0011 driver. Three unprobed edges changed with it (text.md, UNVERIFIED): (1) a LAMBDA argument is `#VALUE!` per element, after an earlier argument's error (`LEFT(NA(),LAMBDA(x,x))` is now `#N/A`, `ROWS(LEFT({"a","b"},LAMBDA(x,x)))` 1x2); (2) a 1x1 array argument gives a 1x1 array (`TYPE(LEFT({"abc"},2))` 64, `ARRAYTOTEXT(LEFT({"abc"},"x"),1)` "{#VALUE!}"), as text.md already records for TEXTSPLIT; (3) VALUETOTEXT with an array value and a scalar error format is an array of that error (`ROWS(VALUETOTEXT({"a",1},NA()))`). Also probe a legacy (pre-dynamic-array) `=RIGHT(A1:A3)`, which now gets the `@` (the old static arm required two arguments). Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case with these cells and oracle values lands under eval/suites/probes/cases/.
- [ ] text.md's entry is VERIFIED or corrected, and any mismatch has a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (a mismatch gets its own task).
