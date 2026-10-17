---
id: T-1763
title: "A call of a defined LAMBDA name prints the defined name's spelling"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
FORMULATEXT of a free call whose name is a defined name prints the defined name's spelling, whatever case the call used.

## Context
Excel: `OtherFn` defined, `=otherfn(1)` then `=OTHERFN(1)` both print `=OtherFn(1)`; ours prints the first typed spelling. Failing cells: probes/unknown-function-name-reach Sheet1!B20, B21. Semantics: docs/excel-semantics/references.md "Unknown function call names" (T-1756); registry `CallNameSpellings` (T-1719).

## Acceptance criteria
- [ ] Sheet1!B20 and B21 of probes/unknown-function-name-reach pass; B1-B3 keep passing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other cells of probes/unknown-function-name-reach.

## Outcome
`CallNameSpellings::apply` takes the workbook's defined names; a free callee matching one (case-insensitively) prints the defined name's spelling. B20 and B21 of probes/unknown-function-name-reach now pass (baseline: two failures fewer, 0 regressions). Defined names used as values and name scope are unprobed.
