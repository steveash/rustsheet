---
id: T-1207
title: "Probe COUNTBLANK of names whose body is an IF, CHOOSE or LET returning a scalar"
milestone: M1
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for COUNTBLANK of a defined name whose body is an IF, CHOOSE or LET that returns a scalar or a reference, with the engine matching.

## Context
T-1199 (probes/countblank-formula-names) covers error, scalar and reference bodies. IF, CHOOSE and LET bodies count as may-be-multi-area (`Model::is_multi_area`) and take the union path in `fn_countblank`, so their scalar result is unprobed: add names `=IF(TRUE,5,6)`, `=CHOOSE(1,5,6)`, `=LET(x,5,x)` and reference-returning forms (`=IF(TRUE,Sheet1!$B$1:$B$3)`) to a new probe case.

## Acceptance criteria
- [ ] Probe cells with excel-graph sidecar; references.md updated; engine matches or mismatches baselined.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Evaluating the name body once instead of twice.
