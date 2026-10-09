---
id: T-0299
title: "Probe lookup picks when the index argument is INDEX/IF/CHOOSE/OFFSET/a name (follow-up of T-0295)"
milestone: M2
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for whether a VLOOKUP/HLOOKUP pick reaches aggregates as a reference when the index argument is a reference-returning expression rather than a plain cell reference.

## Context
T-0295 gates the reference pick on `LookupIndexArg::Reference`, i.e. any index that evaluates to a range, but only plain and parenthesized cell references and `INT(ref)` are probed (probes/lookup-pick-aggregates P1-P6, Q1-Q6). Unprobed: `INDEX(G5:G5,1)`, `IF(TRUE,G5)`, `CHOOSE(1,G5)`, `OFFSET`, a defined name, a multi-cell range. Use the excel-probe skill.

## Acceptance criteria
- [ ] A probe case covers `COUNT(HLOOKUP(42,F5:G6,<index>))` for each shape above (picking FALSE) and references.md records the results; the engine matches or the mismatch is filed as a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes/lookup-pick-index-shapes (40 cells, 37 match the engine); results in references.md. Mismatches filed as T-1286. Eval delta: three new failing cells in a new probe case.
