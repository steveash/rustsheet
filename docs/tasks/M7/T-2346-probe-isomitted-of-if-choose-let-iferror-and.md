---
id: T-2346
title: "Probe ISOMITTED of IF, CHOOSE, LET, IFERROR and LAMBDA-call results over an omitted parameter"
milestone: M7
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Establish with the Excel oracle what `ISOMITTED` gives for `IF(TRUE,y)`, `CHOOSE(1,y)`, `LET(z,y,z)`, `IFERROR(y,0)` and a LAMBDA call wrapping LOOKUP, where `y` is an omitted optional LAMBDA parameter.

## Context
T-2317 made ISOMITTED(LOOKUP(..)) FALSE (gen-m7/2c5ea078f25443a2 Calc!A1087) for a LOOKUP call only (`fn_isomitted`); the pass-through cases are unprobed. See docs/excel-semantics/arrays.md ISOMITTED entry (Tasks T-1627..). Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case with Excel values for each form exists under eval/suites/probes.
- [ ] The arrays.md ISOMITTED entry states the results and `fn_isomitted` matches them.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
