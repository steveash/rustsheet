---
id: T-1149
title: "Metamorphic generator family: SUM((r1,r2)) equals SUM(r1,r2) for same-sheet references"
milestone: M1
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A generated suite checks that a union argument sums like the same references passed as separate
arguments: `SUM((r1,r2,...))` = `SUM(r1,r2,...)` for same-sheet, non-3-D references, so a
regression in the multi-area walk shows up without new oracle runs.

## Context
SPEC-0004 test plan, deferred from T-1062: probe row 1 confirmed the relation (probes/multiarea-agg
H1:H3, overlap counted twice, docs/excel-semantics/references.md "Multi-area references in
functions"). The gen-m1 policy (`tools/src/rustsheet_tools/gen/operators_refs.py`) allows no
function names and the generator has no metamorphic phase yet (docs/architecture/synthetic-generator.md),
so the family needs either a metamorphic check in the eval harness (two cells of one case must agree,
no oracle) or a new small policy whose expectations are the paired cell's value. Pick the one that
fits docs/architecture/synthetic-generator.md and record the choice there.

## Acceptance criteria
- [ ] A seeded family of at least 50 pairs `SUM((r1,r2[,r3]))` / `SUM(r1,r2[,r3])` over a data sheet
      (single cells, ranges, overlapping areas, whole columns), checked by `make eval` or `make check`.
- [ ] Breaking `Model::for_each_cell_in_areas` (e.g. skipping the second area) fails the check.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cross-sheet and 3-D operands (they are errors, not a metamorphic pair).
- AREAS(u) = operand count (T-1063's half of the test plan).
