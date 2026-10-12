---
id: T-1398
title: "Probe the stored xlsx form of a redundant @ with a written group (=@(A1:A3))"
milestone: M1
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know how Excel stores, in the xlsx `<f>` text, an `@` whose operand has a written group and whose `@` is redundant in a scalar context: `=@(A1:A3)`, `=@((A1:A3))` and `=SIN(@(A1:A3))`. Is it `_xlfn.SINGLE((A1:A3))`, `(A1:A3)` or something else?

## Context
T-1343 folded the `@` group cases into `Node::Group`. The engine now exports `=@(A1:A3)` at the root as `_xlfn.SINGLE((A1:A3))`, which round-trips. `remove_redundant_implicit_intersection` (parser/static_analysis.rs) drops a redundant `@` only when its child is not a group. Without the group, `=@A1:A3` is stored as `A1:A3`. docs/excel-semantics/xlsx-storage.md "Written parentheses are stored as written" marks this UNVERIFIED. The method is the one T-1244 used: enter the formulas through the Graph range API, let Excel for the web save, and read the `<f>` text.

## Acceptance criteria
- [ ] The stored `<f>` text of each formula above is recorded in xlsx-storage.md with evidence, and the UNVERIFIED note is resolved.
- [ ] If the engine differs, a follow-up engine task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the exporter.
