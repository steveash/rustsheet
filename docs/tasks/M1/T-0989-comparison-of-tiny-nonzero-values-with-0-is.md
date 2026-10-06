---
id: T-0989
title: "Comparison of tiny nonzero values with 0 is not tolerant (EXP(-708)=0 is FALSE)"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Comparing a tiny nonzero value with 0 follows Excel: `=EXP(-708)=0` is FALSE.

## Context
Evidence probes/subnormal-function-results C4-C6, C60-C62, C65, C69, C73-C75 (T-0189): Excel says `(EXP(-745))=0`, `(EXP(-708))=0` and `(EXP(-720))=0` are FALSE while the engine says TRUE, so the engine's ULP comparison tolerance (numbers.md Comparison tolerance) must not equate a tiny value with exactly 0. EXP(-745) and EXP(-720) are subnormal but not flushed in Excel (numbers.md). Check B5 too: the engine prints EXP(-708) with 15 digits, Excel 14 (T-0199).

## Acceptance criteria
- [ ] probes/subnormal-function-results C4-C6, C60-C62, C65, C69, C73-C75 pass; baseline shrinks; no regression in compare-tolerance cases.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
