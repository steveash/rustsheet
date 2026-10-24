---
id: T-2602
title: "Revise SPEC-0018 from the T-2592 probe: MAP and friends are not errors, nested cells nest again"
milestone: M7
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
SPEC-0018's rules 5 and 7, its representation (one level) and its engine tasks (T-2593, T-2594) match what probes/t2592-nested-groupby-cells shows.

## Context
T-2592 found: MAP/BYROW/BYCOL/REDUCE hand the LAMBDA each nested cell whole (no error); a reference-body LAMBDA still displays `#VALUE!` but INDEX reads its value (TYPE 1); a two-parameter LAMBDA, a computed 1x1 result and subtotal cells are nested; nested cells nest again in a two-level GROUPBY; SUM/COUNT/COUNTA flatten, AVERAGE is #N/A, ARRAYTOTEXT #CALC!, TEXTJOIN/CONCAT #VALUE!. See docs/excel-semantics/arrays.md.

## Acceptance criteria
- [ ] SPEC-0018 rules and design revised from the probe (including whether one level is enough, and rule 4's top-left read against SUM/COUNT/COUNTA flattening); T-2593 and T-2594 bodies updated to match.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-2593, T-2594).
