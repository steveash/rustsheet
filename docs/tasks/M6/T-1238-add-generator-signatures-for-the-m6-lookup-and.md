---
id: T-1238
title: "Add generator signatures for the M6 lookup and reference functions"
milestone: M6
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
`rustsheet-gen --milestone M6` can generate formulas for the 19 in-scope M6 lookup and reference functions: each has a full signature (`returns`, `params` with `mode`, `type`, `domain`, `prior`).

## Context
Split from T-1090, which signed the 20 M6 date and time functions. Left: ADDRESS, AREAS, CHOOSE, CHOOSECOLS,
CHOOSEROWS, COLUMN, COLUMNS, FORMULATEXT, GETPIVOTDATA, HYPERLINK, INDIRECT, LOOKUP, OFFSET, ROW, ROWS, TRANSPOSE,
TRIMRANGE, XLOOKUP, XMATCH. Take argument modes from oracle evidence (excel-probe skill), as T-1090 did with
probes/date-argument-modes. Watch for: OFFSET and INDIRECT are volatile (mark `volatile: true` so the generator skips
them, or decide otherwise with evidence); functions that spill (TRANSPOSE, CHOOSECOLS, CHOOSEROWS, TRIMRANGE) can
overwrite the next generated formula, so check how the generator handles spilling results (ISFORMULA used `cell: true`);
GETPIVOTDATA needs a pivot table; reference params (AREAS, ROW, COLUMN, FORMULATEXT) may make excel-graph refuse a
literal (as ISFORMULA did). Extend test_generator.py's `emitted` checks.

## Acceptance criteria
- [ ] Every `scope: in` M6 lookup entry (no `oracle: none`) has a signature whose required-argument count matches excel-functions.tsv, or a recorded reason it cannot be generated.
- [ ] A seeded `rustsheet-gen --milestone M6` sample is all accepted by excel-graph.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
