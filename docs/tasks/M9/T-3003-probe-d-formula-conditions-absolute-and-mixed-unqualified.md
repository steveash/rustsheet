---
id: T-3003
title: "Probe D* formula conditions: absolute and mixed unqualified references, one-absolute-row ranges, context functions (T-2617 leftovers)"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
Excel evidence settles the D-function formula-condition cases T-2617 implemented by extension without a probe, and ours
follows it.

## Context
T-2617 (probes/db-computed-criteria-other-sheet; criteria.md "D* computed criteria and an unknown header in the gen-m9
database sweep (T-2258)") found that a condition's unqualified references read the database's sheet, and that a moved
relative reference above the first record or outside the database's columns fails the record. `condition_for_records`
and `reads_records` in crates/rustsheet-engine/src/functions/database.rs extend this, unprobed, to:
- an unqualified absolute (`=$B$2>50`) or mixed (`=$B2>50`, `=B$2>50`) reference: moved to the database's sheet like a
  relative one;
- a range with one absolute-row end (`=SUM($B$1:B2)>3`): checked by the larger row of the two ends;
- a range whose columns lie wholly outside the database: fails;
- references the parser does not hold as plain cells: `INDIRECT("B2")`, `OFFSET(B2,0,0)`, a defined name with a relative
  reference, `CELL("address")` with no reference, `SHEET()`, a 3-D span, a whole column (`B:B`), and `ROW()` reached
  through a LAMBDA: ours evaluates them on the criteria sheet at the moved row.
Reuse the probe's layout (Crit/Data sheets, 2^row ids summed by DSUM).

## Acceptance criteria
- [ ] A probe case with excel-graph values for every form above is committed under eval/suites/probes.
- [ ] criteria.md records each result; ours is fixed here if small, else a functions-criteria task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The relative-reference rules T-2617 verified.
