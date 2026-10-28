---
id: T-3002
title: "Probe D* computed criteria with mixed absolute references and evaluation-time references (T-2617 leftovers)"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence decides the D* computed-criterion cases T-2617 left unprobed, so `reads_records` and
`retarget_unqualified_references` rest on evidence everywhere they decide a match.

## Context
- T-2617 (probes/db-computed-criteria-cross-sheet; criteria.md "D* computed criteria and an unknown header in the
  gen-m9 database sweep (T-2258)") showed that a computed criterion's unprefixed references read the database's
  sheet, and that a record matches only when every moved relative reference onto that sheet lies in the database's
  record area (a range must overlap it). Absolute `$B$5` is exempt.
- Unprobed, ours (`reads_records` in functions/database.rs) exempts a reference with an absolute row and a relative
  column (`B$5`), and checks a range with one absolute row by its moved other corner (`SUM(B$5:B6)`, `SUM($B5:B6)`).
- Unprobed, only parsed references are retargeted: `INDIRECT("B2")`, `OFFSET(B2,0,0)`, `ROW()`/`COLUMN()` with no
  argument and a structured `[@Col]` reference in a criterion on another sheet keep the criteria cell's sheet.
- Evidence for `ROW()` exists: the duplicate T-2617 run (closed PR #2127, branch
  agent/research/T-2617-d-computed-criteria-other-sheet, probes/db-computed-criteria-other-sheet Calc A138-A149,
  excel-graph 2026-10-28) found `ROW()` with no argument is the criteria cell's own row for every record (`=ROW()=2`
  on Crit row 2 matches every record of Data!A1:C7, Data!A4:C10 and Crit!A10:C16; `=ROW()>2` and `=ROW()<2` none).
  That branch's case and oracle can be copied, and its `condition_for_records` pins `ROW()` that way.
- Use T-2617's design: field cells holding 2^(row-1) so a DSUM is the set of matched records, over Same!A5:C12 and a
  Data/Db pair. Implementation detail to revisit when touching the code: the cross-sheet path clones the criterion's
  AST once per record (`Arc::make_mut`); retargeting once per criteria cell would avoid it.

## Acceptance criteria
- [ ] A probe case with oracle values for `B$5>0`, `$B5>0`, `SUM(B$5:B6)>0`, `SUM($B5:B6)>0` (same sheet) and
      `INDIRECT("B2")`, `OFFSET(B2,0,0)`, `ROW()`, `[@Col]` conditions (criteria on another sheet) is committed under
      eval/suites/probes.
- [ ] criteria.md records the result; ours is changed to match, or a functions-criteria task is filed for it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The rules T-2617 verified.
