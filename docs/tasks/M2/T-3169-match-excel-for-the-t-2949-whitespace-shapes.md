---
id: T-3169
title: "Match Excel for the T-2949 whitespace shapes (LAMBDA and LET names, intersection LF, leading TAB/LF in stored text)"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
FORMULATEXT prints the shapes probed in T-2949 as Excel does.

## Context
Evidence and rules: docs/excel-semantics/syntax.md "Remaining whitespace shapes (T-2949)"; cases probes/t2949-typed-* and t2949-stored-*. Check which already match (`make eval-show`) and fix the differences: whitespace before a LAMBDA/LET name kept and after it dropped, an intersection separator's line feed kept, a stored leading TAB or LF dropped.

## Acceptance criteria
- [ ] Every t2949 probe cell passes (or its difference is recorded as a DEVIATION).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
