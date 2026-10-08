---
id: T-0292
title: "Probe VLOOKUP/HLOOKUP results in aggregates and INDEX past its range as an intersection operand"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for whether VLOOKUP's (and HLOOKUP's) picked cell reaches an aggregate as a reference, and for `INDEX(A1:A2,3) A1:A2`, with the references.md entry VERIFIED or corrected.

## Context
T-0235 made both lookups pass the picked cell of a range table to aggregates as a reference (`hlookup_impl`/`vlookup_impl`, functions/util.rs `evaluate_aggregate_arg`). Only HLOOKUP has evidence (gen-m2 60a543744fc4a8f7 A200); VLOOKUP is by analogy and marked UNVERIFIED in docs/excel-semantics/references.md. Use the excel-probe skill.

## Acceptance criteria
- [ ] A probe case covers `COUNT(VLOOKUP(..))` picking FALSE, `SUM(VLOOKUP(..))` picking text "5", `MAX(HLOOKUP(..))` picking text, the same over an array table (`{..}`), and `INDEX(A1:A2,3) A1:A2`; the engine matches or the mismatch is filed as a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
