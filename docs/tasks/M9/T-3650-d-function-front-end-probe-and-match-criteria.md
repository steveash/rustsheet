---
id: T-3650
title: "D-function front end: probe and match criteria block grammar, field resolution and error precedence across all 12"
milestone: M9
lane: functions-criteria
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
All twelve D-functions (DAVERAGE, DCOUNT, DCOUNTA, DGET, DMAX, DMIN, DPRODUCT, DSTDEV, DSTDEVP, DSUM, DVAR, DVARP) match Excel on the shared front end: the criteria block grammar, how `field` is resolved, and which error wins, fixed once in the shared code.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md, branch `claude/sharp-mendel-13x2km`). It names a class of input that fails, never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the class, record that in the Outcome and close the task.

The holdout themes note reports identical failures across all twelve D-functions on criteria-range grammar
(wildcards, comparison operators, an empty or unmatched criteria block), on resolving `field` (text versus position,
out of range), and on error precedence, with Excel giving `#VALUE!` where ours gives `#N/A` and numbers against `#N/A`.
Identical failures point at the shared front end (functions/database.rs, criteria through functions/criteria.rs).
Done: T-0205, T-0206, T-1667, T-2235 (criteria grammar pieces), T-1389 (field and criteria lifting), T-1948, T-2516
(DGET error field), T-2525; criteria.md "D-function plain-text equality matches by prefix", "Criteria kinds in
COUNTIF and D-functions", "Database edge-pool smoke check (T-2249)". Open: T-3129 (an error `field` kind only),
T-3002 (computed criteria with mixed references), T-2518 (security cap). Not swept: a criteria block of only headers,
headers with no row, a header absent from the database, a blank header cell, duplicate headers, a header that
matches only by case or with spaces, multi-row OR blocks with blank rows, a criteria range overlapping the database;
`field` as text matching no header, numeric text, 0, negative, fractional (truncation), past the column count, a
boolean, a blank cell, a multi-cell range; a database of only a header row; and errors in the database, the field
and the criteria at once (which wins). Probe each shape through all twelve functions so a front-end rule shows as
the same result family-wide.

## Acceptance criteria
- [ ] A probe case (excel-graph) covers the shapes above through all twelve D-functions.
- [ ] The engine matches every probed cell, through one change in the shared front end (no per-function special case).
- [ ] criteria.md states the grammar, field-resolution and error-order rules found.
- [ ] T-3129's field kinds and T-3002's computed criteria are left to them.
- [ ] Every quirk found is written up in docs/excel-semantics/criteria.md in the same PR, with its probe cells cited.
- [ ] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- Accumulation and last-digit work in DAVERAGE/DVAR (done in T-1054, T-1618 and others).
