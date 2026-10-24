---
id: T-2564
title: "Probe the automatic @ on a one-sheet span range in a legacy-import workbook"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Confirm in Excel that a legacy-import (no dynamic-array flag) `SIN(Sheet1:Sheet1!A1:A10)` takes the automatic `@` as the plain range does, or correct the rule.

## Context
Split from T-2357: the engine puts `@` on a one-sheet span range (T-2351; test_add_implicit_intersection.rs) and now writes it as `SIN(@Sheet1!A1:A10)`. No oracle probe covers the legacy-import case; docs/excel-semantics/references.md "A scalar function over a one-sheet span's range".

## Acceptance criteria
- [ ] A FORMULATEXT cell for `=@Data1:Data1!A1` (now `=@Data1!A1`, unprobed) is in the same probe.
- [ ] A legacy-import probe (formula stored without the dynamic-array flag) shows Excel's value and FORMULATEXT; the unit test and references.md match it.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
