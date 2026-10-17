---
id: T-1767
title: "Spec: one shared lifting of scalar function arguments over arrays and ranges (stat, financial, engineering, date, lookup)"
milestone: M9
lane: engine
status: todo
priority: P1
size: S
tier: opus
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
depends_on: []
---

## Goal
A spec (SPEC-0011) for one table-driven way to lift scalar-parameter functions over array and range arguments, so the statistical, financial, engineering, date and lookup families lift with one implementation instead of per-family copies.

## Context
Owner-directed work (2026-10-17): array lifting of scalar arguments is the largest remaining probe failure bucket. probes/stat-argument-modes (302 lift cells, T-1290), probes/fin-argument-modes (274, T-1372), probes/eng-argument-modes (about 94, T-1383), plus probes/date-argument-modes (T-1236), probes/lookup-argument-modes (T-1241) and probes/m7-argument-modes (T-1271) all fail the same way: `ROWS(F(..{v;v}..))` is 2 in Excel and `#N/IMPL!` or `#VALUE!` here. The legends are in docs/excel-semantics/statistics.md, financial.md, engineering.md, dates.md, lookup.md and arrays.md ("... function argument modes"). Lifting already exists in several shapes: `NumberOrArray` in cast.rs, `macros.rs`, date_and_time.rs `lift`, mathematical.rs, if_ifs.rs and the broadcast helpers (`bcast_get`, `bcast_idx`, T-1418, T-1122). Directive 3 (one definition per concept) and directive 4 (a spec before an architectural change) require one design before the per-family tasks.

## Acceptance criteria
- [ ] docs/specs/SPEC-0011-scalar-argument-lifting.md: the Excel rules with evidence (which parameters lift over arrays, over ranges, or are taken whole; result shape and broadcasting of mismatched sizes, #N/A padding; error per element vs whole result; the ATP functions that refuse ranges), an inventory of today's lifting helpers, the shared design (per-function argument-kind table, one driver, bounded by check_array_size before any element is computed), migration of the existing helpers, and an ordered rollout of S/M tasks.
- [ ] T-1290, T-1372, T-1383 (and T-1236, T-1241, T-1271 where they fit) get `depends_on` on the implementation task and a Context line pointing to the spec; new tasks filed for the rollout with scripts/tasks.py.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the families (T-1290, T-1372, T-1383 and the rollout tasks).
