---
id: T-1918
title: "Probe legacy N and T over CHOOSE and INDEX shapes beyond the T-1768 probe"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's `@` placement and value for legacy N and T over the CHOOSE and INDEX shapes the T-1768 probe did not cover are established by oracle evidence, and the engine matches them.

## Context
T-1908 leaves no `@` on the call for N/T over CHOOSE with any literal index over plain references and INDEX of a plain reference with any indices (`is_plain_reference`, `leaves_call_whole` in static_analysis/legacy.rs). Only `CHOOSE(1,range)`, `INDEX(range,1)` and `INDEX(range,@range)` are probed. Unprobed: `INDEX(range,{1;2})`, `INDEX(range,0)`, `CHOOSE(2,cell,range)`, boolean CHOOSE index, mixed arms. Ours gives 7 for these; Excel is unconfirmed. Use the excel-probe skill.

## Acceptance criteria
- [ ] A probe case covers those shapes (values and FORMULATEXT), the engine matches or the differences are baselined, and `docs/excel-semantics/references.md` records the result.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
